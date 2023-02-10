package http

import (
	"bufio"
	"net"
	"net/http"

	"github.com/felixge/httpsnoop"
	"github.com/nanobus/nanobus/pkg/logger"
)

// This logic is largely taken from gorilla (https://github.com/gorilla/handlers/blob/master/logging.go)
// which is archived and not taking new PRs.

func LoggingHandler(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		snoop, w := makeLogger(w)
		next.ServeHTTP(w, r)
		logger.Debug(r.Method, "path", r.URL.Path, "status", snoop.status)
	})
}

type responseLogger struct {
	w      http.ResponseWriter
	status int
	size   int
}

func (l *responseLogger) Write(b []byte) (int, error) {
	size, err := l.w.Write(b)
	l.size += size
	return size, err
}

func (l *responseLogger) WriteHeader(s int) {
	l.w.WriteHeader(s)
	l.status = s
}

func (l *responseLogger) Status() int {
	return l.status
}

func (l *responseLogger) Size() int {
	return l.size
}

func (l *responseLogger) Hijack() (net.Conn, *bufio.ReadWriter, error) {
	conn, rw, err := l.w.(http.Hijacker).Hijack()
	if err == nil && l.status == 0 {
		// The status will be StatusSwitchingProtocols if there was no error and
		// WriteHeader has not been called yet
		l.status = http.StatusSwitchingProtocols
	}
	return conn, rw, err
}
func makeLogger(w http.ResponseWriter) (*responseLogger, http.ResponseWriter) {
	logger := &responseLogger{w: w, status: http.StatusOK}
	return logger, httpsnoop.Wrap(w, httpsnoop.Hooks{
		Write: func(httpsnoop.WriteFunc) httpsnoop.WriteFunc {
			return logger.Write
		},
		WriteHeader: func(httpsnoop.WriteHeaderFunc) httpsnoop.WriteHeaderFunc {
			return logger.WriteHeader
		},
	})
}
