package oauth2

import (
	"context"
	"crypto/rand"
	"encoding/base64"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/go-logr/logr"
	"github.com/gorilla/mux"
	"golang.org/x/oauth2"

	"github.com/nanobus/nanobus/actions"
	"github.com/nanobus/nanobus/config"
	"github.com/nanobus/nanobus/resolve"
	"github.com/nanobus/nanobus/runtime"
	"github.com/nanobus/nanobus/transport/routes"
)

type Config struct {
	LoginPath    string   `mapstructure:"loginPath"`
	CallbackPath string   `mapstructure:"callbackPath"`
	ClientID     string   `mapstructure:"clientId" validate:"required"`
	ClientSecret string   `mapstructure:"clientSecret" validate:"required"`
	Endpoint     Endpoint `mapstructure:"endpoint" validate=:"required"`
	RedirectURL  string   `mapstructure:"redirectUrl" validate:"required"`
	Scopes       []string `mapstructure:"scopes"`
	Pipeline     string   `mapstructure:"pipeline"`
	Debug        bool     `mapstructure:"debug"`
}

type Endpoint struct {
	AuthURL  string `mapstructure:"authUrl" validate:"required"`
	TokenURL string `mapstructure:"tokenUrl" validate:"required"`

	// AuthStyle optionally specifies how the endpoint wants the
	// client ID & client secret sent. The zero value means to
	// auto-detect.
	AuthStyle AuthStyle `mapstructure:"authStyle"`
}

type Processor interface {
	LoadPipeline(pl *runtime.Pipeline) (runtime.Runnable, error)
	Pipeline(ctx context.Context, name string, data actions.Data) (interface{}, error)
	Provider(ctx context.Context, namespace, service, function string, data actions.Data) (interface{}, error)
	Event(ctx context.Context, name string, data actions.Data) (interface{}, error)
}

// AuthStyle represents how requests for tokens are authenticated
// to the server.
type AuthStyle int

const (
	// AuthStyleAutoDetect means to auto-detect which authentication
	// style the provider wants by trying both ways and caching
	// the successful way for the future.
	AuthStyleAutoDetect AuthStyle = 0

	// AuthStyleInParams sends the "client_id" and "client_secret"
	// in the POST body as application/x-www-form-urlencoded parameters.
	AuthStyleInParams AuthStyle = 1

	// AuthStyleInHeader sends the client_id and client_password
	// using HTTP Basic Authorization. This is an optional style
	// described in the OAuth2 RFC 6749 section 2.3.1.
	AuthStyleInHeader AuthStyle = 2
)

func (a *AuthStyle) DecodeString(str string) error {
	switch strings.ToLower(str) {
	case "auto-detect":
		*a = AuthStyleAutoDetect
	case "inparams":
		*a = AuthStyleInParams
	case "inheader":
		*a = AuthStyleInParams
	default:
		return fmt.Errorf("unknown auth style %q", str)
	}

	return nil
}

type Auth struct {
	log          logr.Logger
	loginPath    string
	callbackPath string
	config       *oauth2.Config
	processor    Processor
	pipeline     string
	debug        bool
}

func Oauth2() (string, routes.Loader) {
	return "oauth2", Loader
}

func Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (routes.AddRoutes, error) {
	c := Config{
		LoginPath:    "/oauth/login",
		CallbackPath: "/oauth/callback",
	}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var logger logr.Logger
	var processor Processor
	if err := resolve.Resolve(resolver,
		"system:logger", &logger,
		"system:processor", &processor); err != nil {
		return nil, err
	}

	config := &oauth2.Config{
		ClientID:     c.ClientID,
		ClientSecret: c.ClientSecret,
		Endpoint: oauth2.Endpoint{
			AuthURL:   c.Endpoint.AuthURL,
			TokenURL:  c.Endpoint.TokenURL,
			AuthStyle: oauth2.AuthStyle(c.Endpoint.AuthStyle),
		},
		RedirectURL: c.RedirectURL,
		Scopes:      c.Scopes,
	}

	oauth := Auth{
		log:          logger,
		loginPath:    c.LoginPath,
		callbackPath: c.CallbackPath,
		config:       config,
		processor:    processor,
		pipeline:     c.Pipeline,
		debug:        c.Debug,
	}

	return oauth.AddRoutes, nil
}

func (o *Auth) AddRoutes(r *mux.Router) {
	r.HandleFunc(o.loginPath, o.login)
	r.HandleFunc(o.callbackPath, o.callback)
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
		o.log.Error(nil, "invalid oauth google state")
		http.Redirect(w, r, "/", http.StatusTemporaryRedirect)
		return
	}

	token, err := o.config.Exchange(r.Context(), r.FormValue("code"))
	if err != nil {
		o.log.Error(err, "could not exchange authorization code")
		http.Redirect(w, r, "/", http.StatusTemporaryRedirect)
		return
	}

	claims, err := getClaims(token)
	if err != nil {
		o.log.Error(err, "could not parse claims")
		http.Redirect(w, r, "/", http.StatusTemporaryRedirect)
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

	if o.pipeline != "" {
		data := actions.Data{
			"claims":        claims,
			"access_token":  token.AccessToken,
			"token_type":    token.TokenType,
			"expiry":        token.Expiry,
			"refresh_token": token.RefreshToken,
		}
		_, err := o.processor.Pipeline(r.Context(), o.pipeline, data)
		if err != nil {
			o.log.Error(err, "could not process authentication pipeline")
			http.Redirect(w, r, "/", http.StatusTemporaryRedirect)
			return
		}
	}

	setSessionCookie(w, token, claims)
	http.Redirect(w, r, "/", http.StatusTemporaryRedirect)
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

func getClaims(token *oauth2.Token) (map[string]any, error) {
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
	decoded, err := base64.RawURLEncoding.DecodeString(claimsSegment)
	if err != nil {
		return nil, err
	}

	var m map[string]any
	if err := json.Unmarshal(decoded, &m); err != nil {
		return nil, err
	}

	return m, nil
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

	cookie := http.Cookie{Name: "sid", Value: sid, Expires: token.Expiry, Path: "/"}
	http.SetCookie(w, &cookie)

	return nil
}
