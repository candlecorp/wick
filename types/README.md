Wick Types
===
Wick Types is a collection of type definitions for the Wick Framework.

Here are a collection of all of the types that will be used by the Wick Framework to communicate between Wick components and the Wick Host.

## Built-in types:
Alias is provided for `int`, `float`, and `bytes` to simplify development. Just use these if you don't have any particular reason for more granular number specifics.

These charts will be updated as more language codegens are created.

| Type       | Alias   | Description                        | Rust Type          |
| ---------- | ------- | ---------------------------------- | ------------------ |
| `bool`     |         | A boolean value                    | `bool`             |
| `u8`       |         | An unsigned 8-bit integer          | `u8`               |
| `i8`       |         | A signed 8-bit integer             | `i8`               |
| `u16`      |         | An unsigned 16-bit integer         | `u16`              |
| `i16`      |         | A signed 16-bit integer            | `i16`              |
| `u32`      |         | An unsigned 32-bit integer         | `u32`              |
| `i32`      | `int`   | A signed 32-bit integer            | `i32`              |
| `u64`      |         | An unsigned 64-bit integer         | `u64`              |
| `i64`      |         | A signed 64-bit integer            | `i64`              |
| `f32`      | `float` | A 32-bit floating point number     | `f32`              |
| `f64`      |         | A 64-bit floating point number     | `f64`              |
| `char`     |         | A single character                 | `char`             |
| `string`   |         | A string of characters             | `String`           |
| `byte`     |         | A single bytes                     | `u8`               |
| [byte]     | `bytes` | A sequence of bytes                | `Vec<u8>`          |
| `datetime` |         | A date and time in RFC 3339 format | `chrono::DateTime` |
| `duration` |         | A duration of time                 | `chrono::Duration` |

## Collections
| Name    | Alias | Description              | Rust Type       |
| ------- | ----- | ------------------------ | --------------- |
| `array` | `[]`  | A dynamically-sized list | `Vec<T>`        |
| `map`   | `{}`  | A key-value map          | `HashMap<K, V>` |

## Wick Host types
These are the types that will be used by the Wick Host to communicate with the Wick Components when using triggers and resources.

| Name    | Description        |
| ------- | ------------------ |
| [HTTP-Request](http/request.yaml) | A HTTP request. |
| [HTTP-Response](http/response.yaml) | A HTTP response. |