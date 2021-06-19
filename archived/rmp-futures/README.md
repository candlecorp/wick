# rmp-futures - Async Rust MessagePack and MessagePack-RPC

[![Build Status](https://travis-ci.com/tylerwhall/rmp-futures.svg?branch=master)](https://travis-ci.com/github/tylerwhall/rmp-futures)

Async encode/decode for msgpack and msgpack-rpc. Intended to allow
deterministic memory use, making the client responsible for allocation. Since
msgpack is a streamable format, the ability to serialize on the fly as the
writer becomes writable is exposed. Similarly, the reader can be written to
deal with individual encoded objects as they are encountered in the stream.
No memory allocations are performed in the library unless requested, such as
returning a string as a Rust `String` or returning an entire object as a
dynamic `rmpv::Value`.

Data types are compatible with [rmp](https://crates.io/crates/rmp ) and
[rmpv](https://crates.io/crates/rmpv) crates where possible.

## Theory of operation

Every time some data is read and a decision point is reached, such as reaching
the start of a new item, the client gets a new object that represents the state
of the stream and only offers operations that are valid at that point. So a
client can have a static decoding sequence for an expected message with the
storage requirements known up front.

The underlying reader or writer are moved into the decoder or encoder. It is
only possible to get ownership back by performing a sequence of valid
operations on the parser, each returning a new type with its own set of valid
operations. This enforces at compile time that elements are read before
processing the next element, and that encoded messsages are well-formed.

Recursive objects yield recursive types. Example data type progression for
reading an array containing one element: `[true]`

- R: Initial plain async reader
- MsgPackFuture<R>: client wraps reader with MsgPackFuture::new() to indicate that this reader is at the start of a message
- ValueFuture::Array(ArrayFuture<R>): decode() returns an enum
- ArrayFuture<R>: client matches on the enum and destructures the inner ArrayFuture
- MsgPackFuture<ArrayFuture<R>>: client calls next(), unwraps the MsgPackOption. Now we're at the start of a new msg that's an element of an array, as indicated by the data type
- ValueFuture::Boolean(true, ArrayFuture<R>): message decoded as "true"
- ArrayFuture<R>: destructure the Boolean, now the ArrayFuture is ready to read the next element
- MsgPackOption::End(R): ArrayFuture::next() returns an enum indicating end of the array
- R: destructuring the End of the array returns the original reader

Encode and Decode support dynamic `write_value()` and `into_value()` functions
that deal with heap-allocated messages. This is easier to use at the cost of
memory, but also allows easier transition from other libraries that use
`rmpv::Value` such as `rmp-rpc`.


## TODO

- RpcMessage::Request should produce a handle to be used to send the message
  back. Currently the client has to save the id and use the correct one when
  starting the response message.

## License

Licensed under either of

* Apache License, Version 2.0 http://www.apache.org/licenses/LICENSE-2.0
* MIT license http://opensource.org/licenses/MIT

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
