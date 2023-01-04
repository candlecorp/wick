package dapr_test

import (
	"context"
	"encoding/json"
	"testing"

	"github.com/dapr/go-sdk/service/common"
	"github.com/go-logr/logr"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/nanobus/nanobus/pkg/codec"
	codec_json "github.com/nanobus/nanobus/pkg/codec/cloudevents/json"
	"github.com/nanobus/nanobus/pkg/handler"
	"github.com/nanobus/nanobus/pkg/transport"
	"github.com/nanobus/nanobus/pkg/transport/dapr"
)

type mockService struct {
	common.Service
	topics   map[string]common.TopicEventHandler
	bindings map[string]common.BindingInvocationHandler
}

func (m *mockService) AddTopicEventHandler(sub *common.Subscription, fn common.TopicEventHandler) error {
	m.topics[sub.Topic] = fn
	return nil
}

func (m *mockService) TriggerTopic(ctx context.Context, topic string, e *common.TopicEvent) {
	if fn, ok := m.topics[topic]; ok {
		if _, err := fn(ctx, e); err != nil {
			panic(err)
		}
	}
}

func (m *mockService) AddBindingInvocationHandler(name string, fn common.BindingInvocationHandler) error {
	m.bindings[name] = fn
	return nil
}

func (m *mockService) TriggerBinding(ctx context.Context, name string, e *common.BindingEvent) {
	if fn, ok := m.bindings[name]; ok {
		if _, err := fn(ctx, e); err != nil {
			panic(err)
		}
	}
}

func (m *mockService) Start() error        { return nil }
func (m *mockService) Stop() error         { return nil }
func (m *mockService) GracefulStop() error { return nil }

type mockInvoker struct {
	h      handler.Handler
	input  any
	output any
	err    error

	invocations map[handler.Handler]int
}

func (m *mockInvoker) Reset() {
	m.h = handler.Handler{}
	m.input = nil
}

func (m *mockInvoker) Invoke(ctx context.Context, h handler.Handler, id string, input interface{}, authorization transport.Authorization) (interface{}, error) {
	m.invocations[h] = m.invocations[h] + 1
	m.h = h
	m.input = input
	return m.output, m.err
}

func TestServerCallbacks(t *testing.T) {
	ctx := context.Background()
	m := &mockService{
		topics:   make(map[string]common.TopicEventHandler),
		bindings: make(map[string]common.BindingInvocationHandler),
	}
	mi := &mockInvoker{
		invocations: make(map[handler.Handler]int),
	}
	codecs := codec.Codecs{
		"json": codec_json.NewCodec(&codec_json.Config{
			SpecVersion: "1.0",
		}),
	}

	handlerTopicV1 := handler.Handler{
		Interface: "PubSub",
		Operation: "testv1",
	}
	handlerTopicV2 := handler.Handler{
		Interface: "PubSub",
		Operation: "testv2",
	}
	handlerBinding := handler.Handler{
		Interface: "Binding",
		Operation: "test",
	}

	s, err := dapr.NewServer(logr.Discard(), m, mi.Invoke, codecs, &dapr.DaprServerV1Config{
		Subscriptions: []dapr.Subscription{
			{
				Pubsub: "test",
				Topic:  "test",
				Codec:  dapr.CodecRef("json"),
				// Default handler
				Handler: &handlerTopicV1,
				// Routing based on type
				Types: map[string]handler.Handler{
					"test.v2": handlerTopicV2,
				},
			},
		},
		Bindings: []dapr.Binding{
			{
				Name:    "test",
				Codec:   dapr.CodecRef("json"),
				Handler: handlerBinding,
			},
		},
	})
	require.NoError(t, err)

	sendTopic := func(data []byte) {
		m.TriggerTopic(ctx, "test", &common.TopicEvent{
			RawData: data,
		})
	}

	sendBinding := func(data []byte) {
		m.TriggerBinding(ctx, "test", &common.BindingEvent{Data: data})
	}

	tests := []struct {
		name    string
		payload any
		testFn  func(data []byte)
	}{
		{
			name: "Test default route",
			payload: map[string]any{
				"type": "test.v1",
				"data": `Hello, Test!`,
			},
			testFn: sendTopic,
		},
		{
			name: "Test routing by type",
			payload: map[string]any{
				"type": "test.v2",
				"data": `Hello, Test!`,
			},
			testFn: sendTopic,
		},
		{
			name: "Test binding",
			payload: map[string]any{
				"type": "test",
				"data": `Hello, Test!`,
			},
			testFn: sendBinding,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			rawData, err := json.Marshal(tt.payload)
			require.NoError(t, err)
			tt.testFn(rawData)
			assert.Equal(t, tt.payload, mi.input)
			mi.Reset()
		})
	}

	// Check handler invocation counts
	assert.Equal(t, 1, mi.invocations[handlerTopicV1])
	assert.Equal(t, 1, mi.invocations[handlerTopicV2])
	assert.Equal(t, 1, mi.invocations[handlerBinding])

	require.NoError(t, s.Listen())
	require.NoError(t, s.Close())
}
