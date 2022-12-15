# Quic-Rpc

A streaming rpc system based on quic

[<img src="https://img.shields.io/badge/github-quic_rpc-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20" >][repo link] [![Latest Version]][crates.io] [![Docs Badge]][docs.rs] ![license badge] [![status badge]][status link]

[Latest Version]: https://img.shields.io/crates/v/quic-rpc.svg
[crates.io]: https://crates.io/crates/quic-rpc
[Docs Badge]: https://img.shields.io/badge/docs-docs.rs-green
[docs.rs]: https://docs.rs/quic-rpc
[license badge]: https://img.shields.io/crates/l/quic-rpc
[status badge]: https://github.com/n0-computer/quic-rpc/actions/workflows/rust.yml/badge.svg
[status link]: https://github.com/n0-computer/quic-rpc/actions/workflows/rust.yml
[repo link]: https://github.com/n0-computer/quic-rpc

## Goals

### Interaction patterns

Provide not just request/response RPC, but also streaming in both directions, similar to [grpc].

- 1 req -> 1 res
- 1 req, update stream -> 1 res
- 1 req -> res stream
- 1 req, update stream -> res stream

It is still a RPC system in the sense that interactions get initiated by the client.

### Transports

- memory transport with very low overhead. In particular, no ser/deser, currently using [flume]
- quic transport via the [quinn] crate
- transparent combination of the above

### API

- The API should be similar to the quinn api. Basically "quinn with types".

## Non-Goals

- Cross language interop. This is for talking from rust to rust
- Any kind of verisoning. You have to do this yourself
- Making remote message passing look like local async function calls
- Being runtime agnostic. This is for tokio

## Example

[computation service](https://github.com/n0-computer/quic-rpc/blob/main/tests/math.rs)

[quinn]: https://docs.rs/quinn/
[flume]: https://docs.rs/flume/
[grpc]: https://grpc.io/
