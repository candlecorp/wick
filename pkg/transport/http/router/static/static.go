/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

//go:generate apex generate
package static

import (
	"context"
	"io"
	"mime"
	"net/http"
	"path/filepath"
	"sort"
	"time"

	"github.com/go-logr/logr"
	"github.com/gorilla/mux"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	nanohttp "github.com/nanobus/nanobus/pkg/transport/http"
	"github.com/nanobus/nanobus/pkg/transport/http/router"
)

func StaticV1Loader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (router.Router, error) {
	c := StaticV1Config{}
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var logger logr.Logger
	if err := resolve.Resolve(resolver,
		"system:logger", &logger); err != nil {
		return nil, err
	}

	return NewV1(logger, c), nil
}

func NewV1(log logr.Logger, config StaticV1Config) router.Router {
	return func(r *mux.Router, address string) error {
		sort.Slice(config.Paths, func(i, j int) bool {
			return len(config.Paths[i].Path) > len(config.Paths[j].Path)
		})
		for _, path := range config.Paths {
			log.Info("Serving static files",
				"path", path.Path,
				"dir", path.Dir,
				"file", path.File,
				"strip", path.Strip)
			var handler http.Handler
			if path.Dir != nil {
				p, _ := filepath.Abs(*path.Dir)
				handler = http.FileServer(http.Dir(p))
			} else if path.File != nil {
				dir := filepath.Dir(*path.File)
				path := filepath.Base(*path.File)
				handler = &ServerFile{fs: http.Dir(dir), path: path}
			}

			if path.Strip != nil {
				handler = http.StripPrefix(*path.Strip, handler)
			}
			r.PathPrefix(path.Path).Handler(nanohttp.LoggingHandler(handler))
		}

		return nil
	}
}

type ServerFile struct {
	fs   http.FileSystem
	path string
}

const sniffLen = 512

func (s *ServerFile) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	f, err := s.fs.Open(s.path)
	if err != nil {
		msg, code := "404 page not found", http.StatusNotFound
		http.Error(w, msg, code)
		return
	}
	defer f.Close()

	d, err := f.Stat()
	if err != nil {
		msg, code := "404 page not found", http.StatusNotFound
		http.Error(w, msg, code)
		return
	}

	name := d.Name()
	sizeFunc := func() (int64, error) { return d.Size(), nil }

	setLastModified(w, d.ModTime())

	code := http.StatusOK

	// If Content-Type isn't set, use the file's extension to find it, but
	// if the Content-Type is unset explicitly, do not sniff the type.
	_, haveType := w.Header()["Content-Type"]
	var ctype string
	if !haveType {
		ctype = mime.TypeByExtension(filepath.Ext(name))
		if ctype == "" {
			// read a chunk to decide between utf-8 text and binary
			var buf [sniffLen]byte
			n, _ := io.ReadFull(f, buf[:])
			ctype = http.DetectContentType(buf[:n])
			_, err := f.Seek(0, io.SeekStart) // rewind to output whole file
			if err != nil {
				http.Error(w, "seeker can't seek", http.StatusInternalServerError)
				return
			}
		}
		w.Header().Set("Content-Type", ctype)
	}

	size, err := sizeFunc()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.WriteHeader(code)

	if r.Method != "HEAD" {
		if _, err := io.CopyN(w, f, size); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
	}
}

func setLastModified(w http.ResponseWriter, modtime time.Time) {
	if !isZeroTime(modtime) {
		w.Header().Set("Last-Modified", modtime.UTC().Format(http.TimeFormat))
	}
}

var unixEpochTime = time.Unix(0, 0)

// isZeroTime reports whether t is obviously unspecified (either zero or Unix()=0).
func isZeroTime(t time.Time) bool {
	return t.IsZero() || t.Equal(unixEpochTime)
}
