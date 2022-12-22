Creating Transports
===
- Create interface definition in `specs/transport/<transport_group>/<transport_name>.axdl`
- In the interface, you will likely need a "handler".  This is the `interface::pipeline` of what will execute upon the action condition of the interface.
- Create `pkg/transport/<transport_group>` folder
- create `pkg/transport/<transport_group>/apex.yaml` file
- run `apex generate` in `pkg/transport/<transport_group>/` directory
- create `package.go` file
- create `<transport_name>.go` file
- create function with the name that is referenced in `generated.go`
- based on the complexity of the interface, there will be different needs but the whole point is to call the `transport.Invoker` method on the `interface` and `operation (pipeline)` that was sent as part of the `handler` config.
- register the transport in `pkg/engine/engine.go`.  Look for `// Transport registration` comment.
- run `just build`
- test using a bus.yaml