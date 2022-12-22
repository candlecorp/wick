/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//go:generate apex generate
package oauth2

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/go-logr/logr"
	"github.com/google/uuid"
	"github.com/gorilla/mux"
	"golang.org/x/oauth2"

	"github.com/nanobus/nanobus/pkg/actions"
	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/runtime"
	"github.com/nanobus/nanobus/pkg/transport/http/router"
)

type HTTPClient interface {
	Do(req *http.Request) (*http.Response, error)
}

type Processor interface {
	LoadPipeline(pl *runtime.Pipeline) (runtime.Runnable, error)
	Interface(ctx context.Context, name, function string, data actions.Data) (interface{}, bool, error)
	Provider(ctx context.Context, name, function string, data actions.Data) (interface{}, bool, error)
}

type Auth struct {
	log          logr.Logger
	httpClient   HTTPClient
	loginPath    string
	callbackPath string
	config       *oauth2.Config
	redirectURL  string
	userInfoURL  string
	processor    runtime.Namespaces
	handler      *handler.Handler
	debug        bool
}

func OAuth2V1Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (router.Router, error) {
	c := OAuth2V1Config{
		LoginPath:    "/oauth/login",
		CallbackPath: "/oauth/callback",
		RedirectURL:  "/",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var logger logr.Logger
	var processor runtime.Namespaces
	var httpClient HTTPClient
	var developerMode bool
	if err := resolve.Resolve(resolver,
		"system:logger", &logger,
		"system:interfaces", &processor,
		"client:http", &httpClient,
		"developerMode", &developerMode); err != nil {
		return nil, err
	}

	scopes := make([]string, 0, len(c.Scopes))
	for _, s := range c.Scopes {
		for _, s := range strings.Split(s, ",") {
			s = strings.TrimSpace(s)
			if s != "" {
				scopes = append(scopes, s)
			}
		}
	}

	config := &oauth2.Config{
		ClientID:     c.ClientID,
		ClientSecret: c.ClientSecret,
		Endpoint: oauth2.Endpoint{
			AuthURL:   c.Endpoint.AuthURL,
			TokenURL:  c.Endpoint.TokenURL,
			AuthStyle: oauth2.AuthStyle(c.Endpoint.AuthStyle),
		},
		RedirectURL: c.CallbackURL,
		Scopes:      scopes,
	}

	oauth := Auth{
		log:          logger,
		httpClient:   httpClient,
		loginPath:    c.LoginPath,
		callbackPath: c.CallbackPath,
		config:       config,
		redirectURL:  c.RedirectURL,
		userInfoURL:  c.Endpoint.UserInfoURL,
		processor:    processor,
		handler:      c.Handler,
		debug:        developerMode,
	}

	return oauth.AddRoutes, nil
}

func (o *Auth) AddRoutes(r *mux.Router, address string) error {
	r.HandleFunc(o.loginPath, o.login)
	r.HandleFunc(o.callbackPath, o.callback)
	return nil
}

func (o *Auth) login(w http.ResponseWriter, r *http.Request) {
	// Create oauthState cookie
	oauthState := generateStateOauthCookie(w)

	u := o.config.AuthCodeURL(oauthState)
	http.Redirect(w, r, u, http.StatusTemporaryRedirect)

}

func (o *Auth) callback(w http.ResponseWriter, r *http.Request) {
	// Read oauthState from Cookie
	oauthState, _ := r.Cookie("oauthstate")

	if oauthState == nil || r.FormValue("state") != oauthState.Value {
		o.log.Error(nil, "Invalid oauth state")
		http.Redirect(w, r, o.redirectURL, http.StatusTemporaryRedirect)
		return
	}

	token, err := o.config.Exchange(r.Context(), r.FormValue("code"))
	if err != nil {
		o.log.Error(err, "could not exchange authorization code")
		http.Redirect(w, r, o.redirectURL, http.StatusTemporaryRedirect)
		return
	}

	claims, err := o.getClaims(token)
	if err != nil {
		o.log.Error(err, "could not parse claims")
		http.Redirect(w, r, o.redirectURL, http.StatusTemporaryRedirect)
		return
	}

	if o.debug {
		o.log.Info("Auth debug info [TURN OFF FOR PRODUCTION]",
			"claims", claims,
			"token_type", token.TokenType,
			"access_token", token.AccessToken,
			"refresh_token", token.RefreshToken,
			"expiry", token.Expiry)
	}

	if o.handler != nil {
		data := actions.Data{
			"claims":        claims,
			"access_token":  token.AccessToken,
			"token_type":    token.TokenType,
			"expiry":        token.Expiry,
			"refresh_token": token.RefreshToken,
		}
		_, ok, err := o.processor.Invoke(r.Context(), *o.handler, data)
		if !ok {
			o.log.Error(err, "could not find handler", "handler", o.handler.String())
			http.Redirect(w, r, o.redirectURL, http.StatusTemporaryRedirect)
			return
		}
		if err != nil {
			o.log.Error(err, "could not process authentication pipeline")
			http.Redirect(w, r, o.redirectURL, http.StatusTemporaryRedirect)
			return
		}
	}

	setSessionCookie(w, token, claims)
	http.Redirect(w, r, o.redirectURL, http.StatusTemporaryRedirect)
}

func (o *Auth) getClaims(token *oauth2.Token) (map[string]any, error) {
	var claimsJSON []byte

	if o.userInfoURL != "" {
		req, err := http.NewRequest("GET", o.userInfoURL, nil)
		if err != nil {
			return nil, err
		}
		req.Header.Add("Authorization", token.TokenType+" "+token.AccessToken)

		res, err := o.httpClient.Do(req)
		if err != nil {
			return nil, err
		}
		defer res.Body.Close()

		claimsJSON, err = io.ReadAll(res.Body)
		if err != nil {
			return nil, err
		}
	} else {
		// Assume we've received a JWT if a user info URL
		// is not configured.
		idx := strings.IndexByte(token.AccessToken, '.')
		if idx < 0 {
			return nil, errors.New("invalid access token")
		}

		skipSegment := token.AccessToken[idx+1:]

		idx = strings.IndexByte(skipSegment, '.')
		if idx < 0 {
			return nil, errors.New("invalid access token")
		}

		claimsSegment := skipSegment[:idx]
		var err error
		claimsJSON, err = base64.RawURLEncoding.DecodeString(claimsSegment)
		if err != nil {
			return nil, err
		}
	}

	var m map[string]any
	if err := json.Unmarshal(claimsJSON, &m); err != nil {
		return nil, err
	}

	// If the claims don't create a session ID,
	// generate one as a UUID.
	if _, exists := m["sid"]; !exists {
		m["sid"] = uuid.New().String()
	}

	return m, nil
}

func generateStateOauthCookie(w http.ResponseWriter) string {
	var expiration = time.Now().Add(20 * time.Minute)

	b := make([]byte, 16)
	rand.Read(b)
	state := base64.URLEncoding.EncodeToString(b)
	cookie := http.Cookie{Name: "oauthstate", Value: state, Expires: expiration}
	http.SetCookie(w, &cookie)

	return state
}

func setSessionCookie(w http.ResponseWriter, token *oauth2.Token, claims map[string]any) error {
	sidIface, ok := claims["sid"]
	if !ok {
		return errors.New("sid claim not present")
	}

	sid, ok := sidIface.(string)
	if !ok {
		return errors.New("sid claim is not a string")
	}

	cookie := http.Cookie{
		Name:    "sid",
		Value:   sid,
		Expires: token.Expiry,
		Path:    "/",
	}
	http.SetCookie(w, &cookie)

	return nil
}
