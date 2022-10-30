package mesh

import (
	"context"
	"encoding/binary"
	"fmt"
	"sync/atomic"

	"github.com/nanobus/iota/go/wasmrs/operations"
	"github.com/nanobus/iota/go/wasmrs/payload"
	"github.com/nanobus/iota/go/wasmrs/rx"
	"github.com/nanobus/iota/go/wasmrs/rx/flux"
	"github.com/nanobus/iota/go/wasmrs/rx/mono"
	"github.com/nanobus/nanobus/compute"
	"go.opentelemetry.io/otel/trace"
	"go.uber.org/multierr"
)

type (
	Mesh struct {
		tracer      trace.Tracer
		instances   map[string]compute.Invoker
		exports     map[string]map[string]*atomic.Pointer[destination]
		unsatisfied []*pending
		done        chan struct{}
	}

	destination struct {
		tracer   trace.Tracer
		instance compute.Invoker
		index    uint32
		name     string
	}

	pending struct {
		instance compute.Invoker
		oper     operations.Operation
	}
)

func New(tracer trace.Tracer) *Mesh {
	return &Mesh{
		tracer:      tracer,
		instances:   make(map[string]compute.Invoker),
		exports:     map[string]map[string]*atomic.Pointer[destination]{},
		unsatisfied: make([]*pending, 0, 10),
		done:        make(chan struct{}),
	}
}

func (m *Mesh) RequestResponse(ctx context.Context, namespace, operation string, p payload.Payload) mono.Mono[payload.Payload] {
	ns, ok := m.exports[namespace]
	if !ok {
		return nil
	}

	ptr, ok := ns[operation]
	if !ok {
		return nil
	}
	dest := ptr.Load()

	return dest.RequestResponse(ctx, p)
}

func (m *Mesh) FireAndForget(ctx context.Context, namespace, operation string, p payload.Payload) {
	ns, ok := m.exports[namespace]
	if !ok {
		return
	}

	ptr, ok := ns[operation]
	if !ok {
		return
	}
	dest := ptr.Load()

	dest.FireAndForget(ctx, p)
}

func (m *Mesh) RequestStream(ctx context.Context, namespace, operation string, p payload.Payload) flux.Flux[payload.Payload] {
	ns, ok := m.exports[namespace]
	if !ok {
		return nil
	}

	ptr, ok := ns[operation]
	if !ok {
		return nil
	}
	dest := ptr.Load()

	return dest.RequestStream(ctx, p)
}

func (m *Mesh) RequestChannel(ctx context.Context, namespace, operation string, p payload.Payload, in flux.Flux[payload.Payload]) flux.Flux[payload.Payload] {
	ns, ok := m.exports[namespace]
	if !ok {
		return nil
	}

	ptr, ok := ns[operation]
	if !ok {
		return nil
	}
	dest := ptr.Load()

	return dest.RequestChannel(ctx, p, in)
}

func (m *Mesh) Close() error {
	var merr error
	for _, inst := range m.instances {
		if err := inst.Close(); err != nil {
			merr = multierr.Append(merr, err)
		}
	}
	close(m.done)
	return merr
}

func (m *Mesh) WaitUntilShutdown() error {
	<-m.done
	return nil
}

func (m *Mesh) Link(inst compute.Invoker) {
	opers := inst.Operations()

	numExported := 0
	for _, op := range opers {
		switch op.Direction {
		case operations.Export:
			ns, ok := m.exports[op.Namespace]
			if !ok {
				ns = make(map[string]*atomic.Pointer[destination])
				m.exports[op.Namespace] = ns
			}
			ptr, ok := ns[op.Operation]
			if !ok {
				ptr = &atomic.Pointer[destination]{}
				ns[op.Operation] = ptr
			}

			ptr.Store(&destination{
				tracer:   m.tracer,
				instance: inst,
				index:    op.Index,
				name:     fmt.Sprintf("%s/%s", op.Namespace, op.Operation),
			})
			numExported++

		case operations.Import:
			if ok := m.linkOperation(inst, op); !ok {
				m.unsatisfied = append(m.unsatisfied, &pending{
					instance: inst,
					oper:     op,
				})
			}
		}
	}

	if numExported > 0 && len(m.unsatisfied) > 0 {
		filtered := m.unsatisfied[:0]
		for _, u := range m.unsatisfied {
			if ok := m.linkOperation(u.instance, u.oper); !ok {
				filtered = append(filtered, u)
			}
		}
		m.unsatisfied = filtered
	}
}

func (m *Mesh) Unsatisfied() []operations.Operation {
	ops := make([]operations.Operation, len(m.unsatisfied))
	for i, pend := range m.unsatisfied {
		ops[i] = pend.oper
	}
	return ops
}

func (m *Mesh) linkOperation(inst compute.Invoker, op operations.Operation) bool {
	ns, ok := m.exports[op.Namespace]
	if !ok {
		return false
	}

	ptr, ok := ns[op.Operation]
	if !ok {
		return false
	}
	dest := ptr.Load()

	switch op.Type {
	case operations.RequestResponse:
		inst.SetRequestResponseHandler(op.Index, dest.RequestResponse)
	case operations.FireAndForget:
		inst.SetFireAndForgetHandler(op.Index, dest.FireAndForget)
	case operations.RequestStream:
		inst.SetRequestStreamHandler(op.Index, dest.RequestStream)
	case operations.RequestChannel:
		inst.SetRequestChannelHandler(op.Index, dest.RequestChannel)
	}

	return true
}

func (d *destination) RequestResponse(ctx context.Context, p payload.Payload) mono.Mono[payload.Payload] {
	ctx, span := d.tracer.Start(ctx, d.name)
	md := p.Metadata()
	if md != nil {
		binary.BigEndian.PutUint32(md, d.index)
	}
	m := d.instance.RequestResponse(ctx, p)
	m.Notify(func(_ rx.SignalType) {
		span.End()
	})
	return m
}

func (d *destination) FireAndForget(ctx context.Context, p payload.Payload) {
	ctx, span := d.tracer.Start(ctx, d.name)
	defer span.End()
	md := p.Metadata()
	if md != nil {
		binary.BigEndian.PutUint32(md, d.index)
	}
	d.instance.FireAndForget(ctx, p)
}

func (d *destination) RequestStream(ctx context.Context, p payload.Payload) flux.Flux[payload.Payload] {
	ctx, span := d.tracer.Start(ctx, d.name)
	md := p.Metadata()
	if md != nil {
		binary.BigEndian.PutUint32(md, d.index)
	}
	f := d.instance.RequestStream(ctx, p)
	f.Notify(func(_ rx.SignalType) {
		span.End()
	})
	return f
}

func (d *destination) RequestChannel(ctx context.Context, p payload.Payload, in flux.Flux[payload.Payload]) flux.Flux[payload.Payload] {
	ctx, span := d.tracer.Start(ctx, d.name)
	md := p.Metadata()
	if md != nil {
		binary.BigEndian.PutUint32(md, d.index)
	}
	f := d.instance.RequestChannel(ctx, p, in)
	f.Notify(func(_ rx.SignalType) {
		span.End()
	})
	return f
}
