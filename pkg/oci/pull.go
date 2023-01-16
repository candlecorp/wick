package oci

import (
	"context"
	"fmt"
	"io"
	"sync"

	"github.com/opencontainers/go-digest"
	ocispec "github.com/opencontainers/image-spec/specs-go/v1"
	"oras.land/oras-go/v2"
	"oras.land/oras-go/v2/content"
	"oras.land/oras-go/v2/content/file"
)

func Pull(reference, target string) (string, error) {
	verbose := false
	var printed sync.Map
	var targetPlatform *ocispec.Platform
	ctx := context.Background()

	repo, err := getRepository(reference)
	if err != nil {
		return "", err
	}

	// Copy Options
	copyOptions := oras.DefaultCopyOptions
	configPath, configMediaType := parseFileReference("", "") // opts.ManifestConfigRef, "")
	if targetPlatform != nil {
		copyOptions.WithTargetPlatform(targetPlatform)
	}
	copyOptions.FindSuccessors = func(ctx context.Context, fetcher content.Fetcher, desc ocispec.Descriptor) ([]ocispec.Descriptor, error) {
		statusFetcher := content.FetcherFunc(func(ctx context.Context, target ocispec.Descriptor) (fetched io.ReadCloser, fetchErr error) {
			if _, ok := printed.LoadOrStore(generateContentKey(target), true); ok {
				return fetcher.Fetch(ctx, target)
			}

			// print status log for first-time fetching
			if err := PrintStatus(target, "Downloading", verbose); err != nil {
				return nil, err
			}
			rc, err := fetcher.Fetch(ctx, target)
			if err != nil {
				return nil, err
			}
			defer func() {
				if fetchErr != nil {
					rc.Close()
				}
			}()
			if err := PrintStatus(target, "Processing ", verbose); err != nil {
				return nil, err
			}
			return rc, nil
		})
		successors, err := content.Successors(ctx, statusFetcher, desc)
		if err != nil {
			return nil, err
		}
		var ret []ocispec.Descriptor
		// Iterate all the successors to
		// 1) Add name annotation to config if configPath is not empty
		// 2) Skip fetching unnamed leaf nodes
		for i, s := range successors {
			// Save the config when:
			// 1) MediaType matches, or
			// 2) MediaType not specified and current node is config.
			// Note: For a manifest, the 0th indexed element is always a
			// manifest config.
			if (s.MediaType == configMediaType || (configMediaType == "" && i == 0 && IsImageManifest(desc))) && configPath != "" {
				// Add annotation for manifest config
				if s.Annotations == nil {
					s.Annotations = make(map[string]string)
				}
				s.Annotations[ocispec.AnnotationTitle] = configPath
			}
			if s.Annotations[ocispec.AnnotationTitle] == "" {
				ss, err := content.Successors(ctx, fetcher, s)
				if err != nil {
					return nil, err
				}
				// Skip s if s is unnamed and has no successors.
				if len(ss) == 0 {
					if _, loaded := printed.LoadOrStore(generateContentKey(s), true); !loaded {
						if err = PrintStatus(s, "Skipped    ", verbose); err != nil {
							return nil, err
						}
					}
					continue
				}
			}
			ret = append(ret, s)
		}
		return ret, nil
	}

	appFile := ""
	dst, err := file.New(target)
	if err != nil {
		return "", err
	}

	pulledEmpty := true
	copyOptions.PreCopy = func(ctx context.Context, desc ocispec.Descriptor) error {
		if _, ok := printed.LoadOrStore(generateContentKey(desc), true); ok {
			return nil
		}
		return PrintStatus(desc, "Downloading", verbose)
	}
	copyOptions.PostCopy = func(ctx context.Context, desc ocispec.Descriptor) error {
		// restore named but deduplicated successor nodes
		successors, err := content.Successors(ctx, dst, desc)
		if err != nil {
			return err
		}
		for _, s := range successors {
			if _, ok := s.Annotations[ocispec.AnnotationTitle]; ok {
				if _, ok := printed.LoadOrStore(generateContentKey(s), true); !ok {
					if err = PrintStatus(s, "Restored   ", verbose); err != nil {
						return err
					}
				}
			}
		}

		name, ok := desc.Annotations[ocispec.AnnotationTitle]
		if !ok {
			name = desc.MediaType
		} else {
			// named content downloaded
			pulledEmpty = false

			// Found the application media type.
			if desc.MediaType == AppMediaType {
				appFile = name
			}
		}
		printed.Store(generateContentKey(desc), true)
		return Print("Downloaded ", ShortDigest(desc), name)
	}

	// Copy
	desc, err := oras.Copy(ctx, repo, repo.Reference.Reference, dst, repo.Reference.Reference, copyOptions)
	if err != nil {
		return "", err
	}
	if pulledEmpty {
		fmt.Println("Downloaded empty artifact")
	}
	fmt.Println("Pulled", reference)
	fmt.Println("Digest:", desc.Digest)
	return appFile, nil
}

// generateContentKey generates a unique key for each content descriptor, using
// its digest and name if applicable.
func generateContentKey(desc ocispec.Descriptor) string {
	return desc.Digest.String() + desc.Annotations[ocispec.AnnotationTitle]
}

// docker media types
const (
	MediaTypeManifest = "application/vnd.docker.distribution.manifest.v2+json"
)

// IsImageManifest checks whether a manifest is an image manifest.
func IsImageManifest(desc ocispec.Descriptor) bool {
	return desc.MediaType == MediaTypeManifest || desc.MediaType == ocispec.MediaTypeImageManifest
}

// ShortDigest converts the digest of the descriptor to a short form for displaying.
func ShortDigest(desc ocispec.Descriptor) (digestString string) {
	digestString = desc.Digest.String()
	if err := desc.Digest.Validate(); err == nil {
		if algo := desc.Digest.Algorithm(); algo == digest.SHA256 {
			digestString = desc.Digest.Encoded()[:12]
		}
	}
	return digestString
}
