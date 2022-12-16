# Scenarios made easier with pipelines

NanoBus does not aim to be a full no/low code solution.
Instead, the goal is to strike the right balance between
imperative and declarative code.

## As a developer, SRE or tech lead I want to...

PubSub / Input Bindings

* publish and consume events encoded in [Avro](https://avro.apache.org) + [Confluent Schema Registry](https://docs.confluent.io/platform/current/schema-registry/index.html)
* use [CloudEvents](https://cloudevents.io) encoded with Avro or Protobuf
* send different versions of events to different handlers
* batch events based on buffer size and interval before sending them to handlers
* transform events to match the data shape my application expects
* invoke actors from domain events
* easily build materialized views from event data

Output Bindings

* map application data to match the expected input of an input binding (portability)
* swap out different component types (state store -> RDBMS)

Resiliency

* apply timeouts to an operation
* apply retries to an operation
* apply circuit breakers to an operation
* specify a fallback operation (DLQ in the case of PubSub)
* inject faults (chaos testing)

Security

* authorize users using a protocol agnostic policy
* control data encryption and tokenization policies for entire records or specific fields that are sensitive

Dev Experience / Efficiency

* reduce or eliminate boilerplate code
* reduce the number of dependencies my application has (SBOM)
* receive data that is already validated
* write application core logic around strongly typed data structures without serialization/deserialization
* trace data as it flows through the application (wire tap)
* mock external dependencies
* share API documentation that is automatically generated for all protocols my service supports
* send data directly from transport to Dapr component (CRUD logic)
* separate infrastructure concepts from code (separation of concerns)
* share API specs with partner teams prior to implementation (API-first)

Integration w/ non-Dapr apps

* invoke endpoints w/ custom TLS config/certificates

## As a product/marketing manager I want to...

Product/Marketing management

* create A/B tests for specific users / personas
