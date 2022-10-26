package paseto

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"aidanwoods.dev/go-paseto"
	"github.com/go-logr/logr"

	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/errorz"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/security/claims"
	"github.com/nanobus/nanobus/transport/filter"
)

type Config struct {
	Audience string `mapstructure:"audience"`
	Issuer   string `mapstructure:"issuer"`

	V4PublicKey    []byte `mapstructure:"v4PublicKey"`
	V4PublicKeyHex string `mapstructure:"v4PublicKeyHex"`
	V3PublicKey    []byte `mapstructure:"v3PublicKey"`
	V3PublicKeyHex string `mapstructure:"v3PublicKeyHex"`
	V2PublicKey    []byte `mapstructure:"v2PublicKey"`
	V2PublicKeyHex string `mapstructure:"v2PublicKeyHex"`

	V4LocalKey    []byte `mapstructure:"v4LocalKey"`
	V4LocalKeyHex string `mapstructure:"v4LocalKeyHex"`
	V3LocalKey    []byte `mapstructure:"v3LocalKey"`
	V3LocalKeyHex string `mapstructure:"v3LocalKeyHex"`
	V2LocalKey    []byte `mapstructure:"v2LocalKey"`
	V2LocalKeyHex string `mapstructure:"v2LocalKeyHex"`
}

type Settings struct {
	Parser paseto.Parser

	// Public purpose
	V4PublicEnabled bool
	V4PublicKey     paseto.V4AsymmetricPublicKey
	V3PublicEnabled bool
	V3PublicKey     paseto.V3AsymmetricPublicKey
	V2PublicEnabled bool
	V2PublicKey     paseto.V2AsymmetricPublicKey

	// Local purpose
	V4LocalEnabled bool
	V4LocalKey     paseto.V4SymmetricKey
	V3LocalEnabled bool
	V3LocalKey     paseto.V3SymmetricKey
	V2LocalEnabled bool
	V2LocalKey     paseto.V2SymmetricKey
}

var (
	ErrV4PublicKeyNotConfigued = errors.New("paseto: a v4 public key is not configured")
	ErrV3PublicKeyNotConfigued = errors.New("paseto: a v3 public key is not configured")
	ErrV2PublicKeyNotConfigued = errors.New("paseto: a v2 public key is not configured")

	ErrV4LocalKeyNotConfigued = errors.New("paseto: a v4 local key is not configured")
	ErrV3LocalKeyNotConfigued = errors.New("paseto: a v3 local key is not configured")
	ErrV2LocalKeyNotConfigued = errors.New("paseto: a v2 local key is not configured")
)

// Paseto is the NamedLoader for the Paseto filter.
func Paseto() (string, filter.Loader) {
	return "paseto", Loader
}

func Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (filter.Filter, error) {
	var c Config
	err := config.Decode(with, &c)
	if err != nil {
		return nil, err
	}

	var logger logr.Logger
	if err := resolve.Resolve(resolver,
		"system:logger", &logger); err != nil {
		return nil, err
	}

	parser := paseto.NewParser()
	if c.Audience != "" {
		parser.AddRule(paseto.ForAudience(c.Audience))
	}
	if c.Issuer != "" {
		parser.AddRule(paseto.IssuedBy(c.Issuer))
	}
	parser.AddRule(paseto.NotExpired())
	parser.AddRule(paseto.ValidAt(time.Now()))

	s := Settings{
		Parser: parser,
	}

	if c.V4PublicKeyHex != "" {
		s.V4PublicEnabled = true
		s.V4PublicKey, err = paseto.NewV4AsymmetricPublicKeyFromHex(c.V4PublicKeyHex)
	} else if len(c.V4PublicKey) > 0 {
		s.V4PublicEnabled = true
		s.V4PublicKey, err = paseto.NewV4AsymmetricPublicKeyFromBytes(c.V4PublicKey)
	}
	if err != nil {
		return nil, fmt.Errorf("invalid v4 public key: %w", err)
	}

	if c.V3PublicKeyHex != "" {
		s.V3PublicEnabled = true
		s.V3PublicKey, err = paseto.NewV3AsymmetricPublicKeyFromHex(c.V3PublicKeyHex)
	} else if len(c.V4PublicKey) > 0 {
		s.V3PublicEnabled = true
		s.V3PublicKey, err = paseto.NewV3AsymmetricPublicKeyFromBytes(c.V3PublicKey)
	}
	if err != nil {
		return nil, fmt.Errorf("invalid v3 public key: %w", err)
	}

	if c.V2PublicKeyHex != "" {
		s.V2PublicEnabled = true
		s.V2PublicKey, err = paseto.NewV2AsymmetricPublicKeyFromHex(c.V2PublicKeyHex)
	} else if len(c.V4PublicKey) > 0 {
		s.V2PublicEnabled = true
		s.V2PublicKey, err = paseto.NewV2AsymmetricPublicKeyFromBytes(c.V2PublicKey)
	}
	if err != nil {
		return nil, fmt.Errorf("invalid v2 public key: %w", err)
	}

	if c.V4LocalKeyHex != "" {
		s.V4LocalEnabled = true
		s.V4LocalKey, err = paseto.V4SymmetricKeyFromHex(c.V4LocalKeyHex)
	} else if len(c.V4LocalKey) > 0 {
		s.V4LocalEnabled = true
		s.V4LocalKey, err = paseto.V4SymmetricKeyFromBytes(c.V4LocalKey)
	}
	if err != nil {
		return nil, fmt.Errorf("invalid v4 local key: %w", err)
	}

	if c.V3LocalKeyHex != "" {
		s.V3LocalEnabled = true
		s.V3LocalKey, err = paseto.V3SymmetricKeyFromHex(c.V3LocalKeyHex)
	} else if len(c.V4LocalKey) > 0 {
		s.V3LocalEnabled = true
		s.V3LocalKey, err = paseto.V3SymmetricKeyFromBytes(c.V3LocalKey)
	}
	if err != nil {
		return nil, fmt.Errorf("invalid v3 local key: %w", err)
	}

	if c.V2LocalKeyHex != "" {
		s.V2LocalEnabled = true
		s.V2LocalKey, err = paseto.V2SymmetricKeyFromHex(c.V2LocalKeyHex)
	} else if len(c.V4LocalKey) > 0 {
		s.V2LocalEnabled = true
		s.V2LocalKey, err = paseto.V2SymmetricKeyFromBytes(c.V2LocalKey)
	}
	if err != nil {
		return nil, fmt.Errorf("invalid v2 local key: %w", err)
	}

	return Filter(logger, &s), nil
}

func Filter(log logr.Logger, settings *Settings) filter.Filter {
	return func(ctx context.Context, header filter.Header) (context.Context, error) {
		authorization := header.Get("Authorization")
		if !strings.HasPrefix(authorization, "Bearer ") {
			return ctx, nil
		}

		// Skip "Bearer "
		tokenString := authorization[7:]

		// Check version prefix "v{1-4}."
		if tokenString[0] != 'v' || tokenString[2] != '.' {
			return ctx, nil
		}

		version := tokenString[1] // '1'-'4'
		tokenPurpose := tokenString[3:]
		i := strings.IndexByte(tokenPurpose, '.')
		if i < 4 {
			return ctx, nil
		}
		purpose := tokenPurpose[0:i] // "local" or "public"

		// Parse the token based on the version and purpose.
		var err error
		var parsedToken *paseto.Token
		switch purpose {
		case "public":
			switch version {
			case '4':
				if !settings.V4PublicEnabled {
					return nil, ErrV4PublicKeyNotConfigued
				}
				parsedToken, err = settings.Parser.ParseV4Public(settings.V4PublicKey, tokenString, nil)
			case '3':
				if !settings.V3PublicEnabled {
					return nil, ErrV3PublicKeyNotConfigued
				}
				parsedToken, err = settings.Parser.ParseV3Public(settings.V3PublicKey, tokenString, nil)
			case '2':
				if !settings.V2PublicEnabled {
					return nil, ErrV2PublicKeyNotConfigued
				}
				parsedToken, err = settings.Parser.ParseV2Public(settings.V2PublicKey, tokenString)
			}
		case "local":
			switch version {
			case '4':
				if !settings.V4LocalEnabled {
					return nil, ErrV4LocalKeyNotConfigued
				}
				parsedToken, err = settings.Parser.ParseV4Local(settings.V4LocalKey, tokenString, nil)
			case '3':
				if !settings.V3LocalEnabled {
					return nil, ErrV3LocalKeyNotConfigued
				}
				parsedToken, err = settings.Parser.ParseV3Local(settings.V3LocalKey, tokenString, nil)
			case '2':
				if !settings.V2LocalEnabled {
					return nil, ErrV2LocalKeyNotConfigued
				}
				parsedToken, err = settings.Parser.ParseV2Local(settings.V2LocalKey, tokenString)
			}
		default:
			return ctx, nil
		}
		if err != nil {
			// Deal with error of token which failed to be
			// validated, or cryptographically verified.
			return nil, errorz.Wrap(err, errorz.Unauthenticated, err.Error())
		}

		if parsedToken != nil {
			if tokenClaims := parsedToken.Claims(); tokenClaims != nil {
				ctx = claims.ToContext(ctx, claims.Claims(tokenClaims))
			}
		}

		return ctx, nil
	}
}
