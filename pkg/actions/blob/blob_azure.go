/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package blob

import (
	"context"
	"errors"
	"fmt"

	"github.com/Azure/azure-sdk-for-go/sdk/azcore/policy"
	"github.com/Azure/azure-sdk-for-go/sdk/azidentity"
	"github.com/Azure/azure-sdk-for-go/sdk/storage/azblob/container"
	"gocloud.dev/blob/azureblob"

	"github.com/nanobus/nanobus/pkg/config"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
	"github.com/nanobus/nanobus/pkg/runtime"
)

type AzureBlobConfig struct {
	ServiceURL       string `mapstructure:"serviceUrl" validate:"required"`
	ContainerName    string `mapstructure:"containerName" validate:"required"`
	CredentialType   string `mapstructure:"credentialType"`
	AccountName      string `mapstructure:"accountName" validate:"required_if=CredentialType sharedKey"`
	AccountKey       string `mapstructure:"accountKey" validate:"required_if=CredentialType sharedKey"`
	ConnectionString string `mapstructure:"connectionString" validate:"required_if=CredentialType connectionString"`
}

// AzureBlob is the NamedLoader for Azure Blob storage.
func AzureBlob() (string, resource.Loader) {
	return "nanobus.resource.azureblob/v1", AzureBlobLoader
}

func AzureBlobLoader(ctx context.Context, with interface{}, resolver resolve.ResolveAs) (interface{}, error) {
	var c AzureBlobConfig
	if err := config.Decode(with, &c); err != nil {
		return nil, err
	}

	var app *runtime.Application
	if err := resolve.Resolve(resolver,
		"system:application", &app); err != nil {
		return nil, err
	}

	client, err := getClient(&c, app)
	if err != nil {
		return nil, err
	}

	// Create a *blob.Bucket.
	return azureblob.OpenBucket(ctx, client, &azureblob.Options{})
}

func getClient(c *AzureBlobConfig, app *runtime.Application) (*container.Client, error) {
	// Set the ApplicationID.
	azClientOpts := &container.ClientOptions{}
	azClientOpts.Telemetry = policy.TelemetryOptions{
		ApplicationID: app.ID,
	}

	containerURL := fmt.Sprintf("%s/%s", c.ServiceURL, c.ContainerName)
	switch c.CredentialType {
	case "default":
		cred, err := azidentity.NewDefaultAzureCredential(nil)
		if err != nil {
			return nil, fmt.Errorf("failed azidentity.NewDefaultAzureCredential: %v", err)
		}
		return container.NewClient(containerURL, cred, azClientOpts)
	case "sharedKey":
		sharedKeyCred, err := container.NewSharedKeyCredential(c.AccountName, c.AccountKey)
		if err != nil {
			return nil, fmt.Errorf("failed azblob.NewSharedKeyCredential: %v", err)
		}
		return container.NewClientWithSharedKeyCredential(containerURL, sharedKeyCred, azClientOpts)
	case "sasViaNone":
		return container.NewClientWithNoCredential(containerURL, azClientOpts)
	case "connectionString":
		return container.NewClientFromConnectionString(c.ConnectionString, c.ContainerName, azClientOpts)
	default:
		return nil, errors.New("internal error, unknown cred type")
	}
}
