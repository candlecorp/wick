package oci

import (
	"context"
	"fmt"
	"path/filepath"
	"regexp"
	"strings"

	ocispec "github.com/opencontainers/image-spec/specs-go/v1"
	"oras.land/oras-go/v2/content/file"
)

const (
	AppMediaType        = "application/vnd.nanobus.app"
	ApexMediaType       = "application/vnd.apexlang.apex"
	ApexConfigMediaType = "application/vnd.apexlang.config"
	WasmMediaType       = "application/wasm"
)

var mediaTypeExts = map[string]string{
	".apex": AppMediaType,
	".wasm": WasmMediaType,
}

var mediaTypeFilenames = map[string]string{
	"apex.yaml": ApexConfigMediaType,
}

var reImageReference = regexp.MustCompile(`(?m)^([a-zA-Z0-9\-\.]+)\/([a-zA-Z0-9\-\.]+)\/([a-zA-Z0-9\-\.]+):([a-zA-Z0-9\-\.]+)$`)

// IsImageReference tests if a string is an OCI image reference.
func IsImageReference(location string) bool {
	return reImageReference.MatchString(location)
}

func loadFiles(ctx context.Context, store *file.Store, annotations map[string]map[string]string, base string, fileRefs []string, verbose bool) ([]ocispec.Descriptor, error) {
	var files []ocispec.Descriptor
	for _, fileRef := range fileRefs {
		filename, mediaType := parseFileReference(fileRef, "")
		if mediaType == "" {
			// Media types based on file extension
			ext := filepath.Ext(filename)
			if mt, ok := mediaTypeExts[ext]; ok {
				mediaType = mt
			}

			// Media types based on entire filename
			f := filepath.Base(filename)
			if mt, ok := mediaTypeFilenames[f]; ok {
				mediaType = mt
			}
		}

		// get shortest absolute path as unique name
		name := filepath.Clean(filename)
		if !filepath.IsAbs(name) {
			name = filepath.ToSlash(name)
		}

		filename = filepath.Join(base, filename)

		if verbose {
			fmt.Println("Preparing", name)
		}
		file, err := store.Add(ctx, name, mediaType, filename)
		if err != nil {
			return nil, err
		}
		if value, ok := annotations[filename]; ok {
			if file.Annotations == nil {
				file.Annotations = value
			} else {
				for k, v := range value {
					file.Annotations[k] = v
				}
			}
		}
		files = append(files, file)
	}
	if len(files) == 0 {
		fmt.Println("Uploading empty artifact")
	}
	return files, nil
}

// parseFileReference parses file reference on unix.
func parseFileReference(reference string, mediaType string) (filePath, mediatype string) {
	i := strings.LastIndex(reference, ":")
	if i < 0 {
		return reference, mediaType
	}
	return reference[:i], reference[i+1:]

}
