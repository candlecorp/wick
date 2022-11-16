## Streamline exposing services with multiple protocols

* [x] HTTP-RPC
  * [x] Mux
  * [x] Pluggable codecs
  * [ ] "Try it" UI
* [ ] gRPC / Proto
  * [ ] Unmarshal
    * [x] nested messages
    * [x] string
    * [x] boolean
    * [x] bytes
    * [x] integers (int64/uint64, int32/uint32)
    * [ ] efficent signed integers (sint64, sint32)
    * [ ] fixed unsigned integers (fixed64, fixed32)
    * [ ] fixed signed integers (sfixed64, sfixed32)
    * [x] floating point (double, float)
    * [x] repeated
    * [x] maps
    * [ ] optional / nullable wrappers
  * [ ] Marshal
    * [ ] nested messages
    * [ ] string
    * [ ] boolean
    * [ ] bytes
    * [ ] integers (int64/uint64, int32/uint32)
    * [ ] efficent signed integers (sint64, sint32)
    * [ ] fixed unsigned integers (fixed64, fixed32)
    * [ ] fixed signed integers (sfixed64, sfixed32)
    * [ ] floating point (double, float)
    * [ ] repeated
    * [ ] maps
    * [ ] optional / nullable wrappers
  * [ ] gRPC proxying
    * [ ] unary
    * [ ] streams
  * [ ] Expose proto spec doc
* [ ] REST / OpenAPI
  * [x] Mux
  * [x] Path parameters
  * [x] Body parameters
  * [ ] Query parameters
    * [x] Single values
    * [ ] Arrays
  * [ ] Content negotiation?
  * [x] Pluggable codecs?
  * [x] Expose OpenAPI spec doc
  * [x] Swagger UI