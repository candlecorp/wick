/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

package dapr_test

import (
	"context"

	dapr "github.com/dapr/go-sdk/client"

	"github.com/nanobus/nanobus/pkg/codec"
	codec_json "github.com/nanobus/nanobus/pkg/codec/json"
	"github.com/nanobus/nanobus/pkg/resolve"
	"github.com/nanobus/nanobus/pkg/resource"
)

type mockClient struct {
	dapr.Client

	// PubSub
	pubsubName  string
	topicName   string
	publishData []byte
	publishErr  error

	// Bindings
	bindingReq *dapr.InvokeBindingRequest
	bindingOut *dapr.BindingEvent
	bindingErr error

	// Delete State
	deleteName string
	deleteKey  string
	deleteEtag *dapr.ETag
	deleteMeta map[string]string
	deleteOpts *dapr.StateOptions

	// Get State
	getName string
	getKey  string
	getItem *dapr.StateItem

	// Save State
	saveName  string
	saveItems []*dapr.SetStateItem

	// Invoke actor
	actorReq  *dapr.InvokeActorRequest
	actorResp *dapr.InvokeActorResponse
	actorErr  error
}

func (m *mockClient) InvokeBinding(ctx context.Context, req *dapr.InvokeBindingRequest) (out *dapr.BindingEvent, err error) {
	m.bindingReq = req
	return m.bindingOut, m.bindingErr
}

func (m *mockClient) PublishEvent(ctx context.Context, pubsubName, topicName string, data any, opts ...dapr.PublishEventOption) error {
	m.pubsubName = pubsubName
	m.topicName = topicName
	m.publishData = data.([]byte)
	return m.publishErr
}

func (m *mockClient) DeleteState(ctx context.Context, storeName, key string, meta map[string]string) error {
	m.deleteName = storeName
	m.deleteKey = key
	m.deleteEtag = nil
	m.deleteMeta = meta
	m.deleteOpts = nil
	return nil
}

func (m *mockClient) DeleteStateWithETag(ctx context.Context, storeName, key string, etag *dapr.ETag, meta map[string]string, opts *dapr.StateOptions) error {
	m.deleteName = storeName
	m.deleteKey = key
	m.deleteEtag = etag
	m.deleteMeta = meta
	m.deleteOpts = opts
	return nil
}

func (m *mockClient) GetStateWithConsistency(ctx context.Context, storeName, key string, meta map[string]string, sc dapr.StateConsistency) (item *dapr.StateItem, err error) {
	m.getName = storeName
	m.getKey = key
	return m.getItem, nil
}

func (m *mockClient) SaveBulkState(ctx context.Context, storeName string, items ...*dapr.SetStateItem) error {
	m.saveName = storeName
	m.saveItems = items
	return nil
}

func (m *mockClient) InvokeActor(ctx context.Context, req *dapr.InvokeActorRequest) (*dapr.InvokeActorResponse, error) {
	m.actorReq = req
	return m.actorResp, m.actorErr
}

func getMockClient(m *mockClient) resolve.ResolveAs {
	r := resource.Resources{
		"dapr": m,
	}
	c := codec.Codecs{
		"json": codec_json.NewCodec(),
	}
	return func(name string, target interface{}) bool {
		switch name {
		case "resource:lookup":
			return resolve.As(r, target)
		case "codec:lookup":
			return resolve.As(c, target)
		}
		return false
	}
}
