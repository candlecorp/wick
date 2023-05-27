---
title: Streaming
---
One of the biggest differentiators between Wick and other WebAssembly frameworks is Wick's adoption of [WasmRS](https://github.com/wasmrs/docs/blob/main/wasmrs.md) and full data streaming support.

## WasmRS
Wick incorporates WasmRS, which implements [reactive streams](https://www.reactive-streams.org/) in WebAssembly modules. This enables asynchronous, bidirectional communication in and out of WebAssembly, expanding the capabilities of Wick. WasmRS is leading the industry and taking the possibilities with WebAssembly to new heights.

### WasmRS Protocol
At the core of WasmRS is a set of methods that allow the host and guest to write RSocket frames to their respective buffers in WebAssembly memory. Language-specific implementations handle the encoding and decoding of these frames, providing a lightweight user experience layer on top and metadata extensions relevant to WebAssembly usage.

Similar to RSocket, WasmRS frames contain a stream ID that allows the destination to differentiate multiple frames for different transactions.

### Benefits
1. **Asynchronous and Bidirectional Communication:** WasmRS allows asynchronous, bidirectional communication between the host and guest, enabling more complex and efficient interactions within WebAssembly applications.
2. **Scalability:** Reactive streams provide a non-blocking, back-pressure mechanism that helps applications scale by efficiently handling large volumes of data.
3. **Multiplexing:** With support for multiple concurrent streams, Wick can handle multiple requests and responses simultaneously, improving overall performance and resource utilization.
4. **Flexible Interaction Models:** WasmRS supports a variety of interaction models, including request/response, fire-and-forget, request stream, and request channel. This flexibility allows developers to choose the most suitable model for their use case, further enhancing the capabilities of Wick applications.
5. **Reactive Programming:** WasmRS brings the power of reactive programming to WebAssembly, enabling developers to build more responsive, resilient, and resource-efficient applications.
6. **Stream Processing:** The ability to process data streams on-the-fly makes Wick an excellent choice for applications that require real-time data processing, such as analytics, monitoring, and event-driven architectures. By leveraging streaming capabilities, developers can create more efficient and high-performing applications that handle continuous data flows.