/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//go:generate apex generate
package jwt

import (
	"context"
	"crypto/ecdsa"
	"crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"encoding/pem"
	"errors"
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/MicahParks/keyfunc"
	"github.com/go-logr/logr"
	"github.com/golang-jwt/jwt/v4"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/errorz"
	"github.com/nanobus/nanobus/pkg/logger"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/security/claims"
	"github.com/nanobus/nanobus/pkg/transport/filter"
)

type Settings struct {
	RSAPublicKey   *rsa.PublicKey
	ECDSAPublicKey *ecdsa.PublicKey
	HMACSecretKey  []byte
	KeyFunc        jwt.Keyfunc
	Debug          bool
}

func JWTV1Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (filter.Filter, error) {
	var c JWTV1Config
	err := config.Decode(with, &c)
	if err != nil {
		return nil, err
	}

	var logger logr.Logger
	var developerMode bool
	if err := resolve.Resolve(resolver,
		"system:logger", &logger,
		"developerMode", &developerMode); err != nil {
		return nil, err
	}

	settings := Settings{
		Debug: developerMode,
	}

	if c.JWKSURL != nil {
		logger.Info("Using JWKS URL for JWT verification")
		// Create the JWKS from the resource at the given URL.
		options := keyfunc.Options{
			Ctx: ctx,
			RefreshErrorHandler: func(err error) {
				logger.Error(err, "There was an error with the jwt.Keyfunc")
			},
			RefreshInterval:   time.Hour,
			RefreshRateLimit:  time.Minute * 5,
			RefreshTimeout:    time.Second * 10,
			RefreshUnknownKID: true,
		}

		kf, err := keyfunc.Get(*c.JWKSURL, options)
		if err != nil {
			return nil, fmt.Errorf("failed to get the JWKS from the given URL.\nError: %w", err)
		}
		settings.KeyFunc = kf.Keyfunc
	}

	var rsaPublicKeyBytes []byte
	if c.RSAPublicKeyFile != nil {
		rsaPublicKeyBytes, err = os.ReadFile(*c.RSAPublicKeyFile)
		if err != nil {
			return nil, fmt.Errorf("cannot read public key file: %w", err)
		}
	} else if c.RSAPublicKeyString != nil {
		rsaPublicKeyBytes = []byte(*c.RSAPublicKeyString)
	}
	if rsaPublicKeyBytes != nil {
		pubPem, _ := pem.Decode(rsaPublicKeyBytes)
		if pubPem == nil {
			return nil, nil
		}
		var parsedKey interface{}
		if parsedKey, err = x509.ParsePKIXPublicKey(pubPem.Bytes); err != nil {
			return nil, err
		}

		var ok bool
		if settings.RSAPublicKey, ok = parsedKey.(*rsa.PublicKey); !ok {
			return nil, errors.New("parsed key was not a RSA public key")
		}
	}

	var ecdsaPublicKeyBytes []byte
	if c.ECDSAPublicKeyFile != nil {
		ecdsaPublicKeyBytes, err = os.ReadFile(*c.ECDSAPublicKeyFile)
		if err != nil {
			return nil, fmt.Errorf("cannot read public key file: %w", err)
		}
	} else if c.ECDSAPublicKeyString != nil {
		ecdsaPublicKeyBytes = []byte(*c.ECDSAPublicKeyString)
	}
	if ecdsaPublicKeyBytes != nil {
		pubPem, _ := pem.Decode(ecdsaPublicKeyBytes)
		if pubPem == nil {
			return nil, nil
		}
		var parsedKey interface{}
		if parsedKey, err = x509.ParsePKIXPublicKey(pubPem.Bytes); err != nil {
			return nil, err
		}

		var ok bool
		if settings.ECDSAPublicKey, ok = parsedKey.(*ecdsa.PublicKey); !ok {
			return nil, errors.New("parsed key was not a ECDSA public key")
		}
	}

	if c.HMACSecretKeyFile != nil {
		settings.HMACSecretKey, err = os.ReadFile(*c.HMACSecretKeyFile)
		if err != nil {
			return nil, fmt.Errorf("cannot read secret key file: %w", err)
		}
		if c.HMACSecretKeyBase64 {
			settings.HMACSecretKey, err = base64.StdEncoding.DecodeString(string(settings.HMACSecretKey))
			if err != nil {
				return nil, fmt.Errorf("cannot base64 decode secret key file: %w", err)
			}
		}
	}

	if settings.KeyFunc == nil {
		settings.KeyFunc = func(token *jwt.Token) (interface{}, error) {
			switch token.Method.(type) {
			case *jwt.SigningMethodRSA:
				return settings.RSAPublicKey, nil
			case *jwt.SigningMethodECDSA:
				return settings.ECDSAPublicKey, nil
			case *jwt.SigningMethodHMAC:
				return settings.HMACSecretKey, nil
			}

			return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
		}
	}

	return Filter(logger, &settings), nil
}

func Filter(log logr.Logger, settings *Settings) filter.Filter {
	return func(ctx context.Context, header filter.Header) (context.Context, error) {
		authorization := header.Get("Authorization")
		if !strings.HasPrefix(authorization, "Bearer ") {
			return ctx, nil
		}

		tokenString := authorization[7:]
		// Check for the prefix of all JWTs.
		if !strings.HasPrefix(tokenString, "ey") {
			return ctx, nil
		}

		token, err := jwt.Parse(tokenString, settings.KeyFunc)
		if err != nil {
			return nil, errorz.Wrap(err, errorz.Unauthenticated, err.Error())
		}

		if token != nil {
			if c, ok := token.Claims.(jwt.MapClaims); ok && token.Valid {
				c := claims.Claims(c)
				ctx = claims.ToContext(ctx, c)

				if settings.Debug {
					logger.Debug("Claims debug info [TURN OFF FOR PRODUCTION]",
						"component", "jwt",
						"claims", c)
				}
			}
		}

		return ctx, nil
	}
}
