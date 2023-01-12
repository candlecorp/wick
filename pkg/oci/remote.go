package oci

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"oras.land/oras-go/v2/registry/remote"
	"oras.land/oras-go/v2/registry/remote/auth"
)

var (
	ErrNotAnOCIImage = errors.New("not an OCI image")
)

type configs struct {
	Auths map[string]struct {
		Username string `json:"username"`
		Password string `json:"password"`
		Email    string `json:"email"`
		Auth     string `json:"auth"`
	} `json:"auths"`
}

func getRepository(reference string) (*remote.Repository, error) {
	repos := make(map[string]auth.Credential)

	homedir, err := os.UserHomeDir()
	if err != nil {
		return nil, err
	}

	configFile := filepath.Join(homedir, ".docker/config.json")
	if _, err := os.Stat(configFile); err == nil {
		contents, err := os.ReadFile(configFile)
		if err != nil {
			fmt.Println("Error reading config file:", err)
			return nil, err
		}

		var config configs
		if err := json.Unmarshal(contents, &config); err != nil {
			fmt.Println("Error parsing config file:", err)
			return nil, err
		}

		for registry, authconfigs := range config.Auths {
			if authconfigs.Username != "" && authconfigs.Password != "" {
				fmt.Println("Adding registry credentials from file: ", registry)
				repos[registry] = auth.Credential{
					Username: authconfigs.Username,
					Password: authconfigs.Password,
				}
			}
		}
	} else if os.IsNotExist(err) {
		fmt.Println(configFile, "file does not exist.")
	} else {
		fmt.Println("Error checking config file:", err)
	}

	registriesString := os.Getenv("OCI_REGISTRIES")

	if registriesString != "" {
		registries := strings.Split(registriesString, ",")
		for _, registry := range registries {
			registry = strings.TrimSpace(registry)

			hostname := os.Getenv(registry + "_HOSTNAME")
			username := os.Getenv(registry + "_USERNAME")
			password := os.Getenv(registry + "_PASSWORD")

			if hostname != "" && username != "" && password != "" {
				if repos[hostname].Username != "" {
					fmt.Println("ENV Registry credentials overwriting config file credentials for: ", registry)
				}
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
