package oci

import (
	"errors"
	"fmt"
	"os"
	"strings"

	"oras.land/oras-go/v2/registry/remote"
	"oras.land/oras-go/v2/registry/remote/auth"
)

var (
	ErrNotAnOCIImage = errors.New("not an OCI image")
)

func getRepository(reference string) (*remote.Repository, error) {
	repos := make(map[string]auth.Credential)
	registriesString := os.Getenv("OCI_REGISTRIES")

	if registriesString != "" {
		registries := strings.Split(registriesString, ",")
		for _, registry := range registries {
			registry = strings.TrimSpace(registry)

			hostname := os.Getenv(registry + "_HOSTNAME")
			username := os.Getenv(registry + "_USERNAME")
			password := os.Getenv(registry + "_PASSWORD")

			if hostname != "" && username != "" && password != "" {
				repos[hostname] = auth.Credential{
					Username: username,
					Password: password,
				}
			}
		}
	}

	repo, err := remote.NewRepository(reference)
	if err != nil {
		return nil, fmt.Errorf("could not access repository %s: %w", reference, err)
	}

	if credential, ok := repos[repo.Reference.Registry]; ok {
		repo.Client = &auth.Client{
			Credential: auth.StaticCredential(repo.Reference.Registry, credential),
		}
	}

	return repo, nil
}
