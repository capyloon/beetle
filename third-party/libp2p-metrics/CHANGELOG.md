# 0.11.0

- Update to `libp2p-dcutr` `v0.8.0`.

- Update to `libp2p-identify` `v0.41.0`.

- Update to `libp2p-relay` `v0.14.0`.

- Update to `libp2p-core` `v0.38.0`.

- Update to `libp2p-swarm` `v0.41.0`.

- Update to `libp2p-ping` `v0.41.0`.

- Update to `libp2p-kad` `v0.42.0`.

- Update to `libp2p-gossipsub` `v0.43.0`.

- Add `protocol_stack` metrics. See [PR 2982].

- Update `rust-version` to reflect the actual MSRV: 1.62.0. See [PR 3090].

- Changed `Metrics::query_result_get_record_ok` from `Histogram` to a `Counter`.
  See [PR 2712].

[PR 2982]: https://github.com/libp2p/rust-libp2p/pull/2982/
[PR 3090]: https://github.com/libp2p/rust-libp2p/pull/3090
[PR 2712]: https://github.com/libp2p/rust-libp2p/pull/2712

# 0.10.0

- Update to `libp2p-swarm` `v0.40.0`.

- Update to `libp2p-dcutr` `v0.7.0`.

- Update to `libp2p-ping` `v0.40.0`.

- Update to `libp2p-identify` `v0.40.0`.

- Update to `libp2p-relay` `v0.13.0`.

- Update to `libp2p-kad` `v0.41.0`.

- Update to `libp2p-core` `v0.37.0`.

- Update to `libp2p-gossipsub` `v0.42.0`.

# 0.9.0

- Update to `libp2p-swarm` `v0.39.0`.

- Update to `libp2p-dcutr` `v0.6.0`.

- Update to `libp2p-ping` `v0.39.0`.

- Update to `libp2p-identify` `v0.39.0`.

- Update to `libp2p-relay` `v0.12.0`.

- Update to `libp2p-kad` `v0.40.0`.

- Update to `libp2p-core` `v0.36.0`.

# 0.8.0

- Update to `libp2p-swarm` `v0.38.0`.

- Update to `libp2p-dcutr` `v0.5.0`.

- Update to `libp2p-ping` `v0.38.0`.

- Update to `libp2p-identify` `v0.38.0`.

- Update to `libp2p-relay` `v0.11.0`.

- Update to `libp2p-kad` `v0.39.0`.

- Track number of connected nodes supporting a specific protocol via the identify protocol. See [PR 2734].

- Update to `libp2p-core` `v0.35.0`.

- Update to `prometheus-client` `v0.18.0`. See [PR 2822].

[PR 2822]: https://github.com/libp2p/rust-libp2p/pull/2761/

[PR 2734]: https://github.com/libp2p/rust-libp2p/pull/2734/

# 0.7.0

- Update to `libp2p-core` `v0.34.0`.

- Update to `libp2p-swarm` `v0.37.0`.

- Update to `libp2p-dcutr` `v0.4.0`.

- Update to `libp2p-ping` `v0.37.0`.

- Update to `libp2p-identify` `v0.37.0`.

- Update to `libp2p-relay` `v0.10.0`.

- Update to `libp2p-kad` `v0.38.0`.

# 0.6.1

- Update `dcutr` events from `libp2p_relay_events` to `libp2p_dcutr_events`, to avoid conflict with `relay` events.

# 0.6.0

- Update to `libp2p-core` `v0.33.0`.

- Update to `libp2p-swarm` `v0.36.0`.

- Update to `libp2p-dcutr` `v0.3.0`.

- Update to `libp2p-ping` `v0.36.0`.

- Update to `libp2p-identify` `v0.36.0`.

- Update to `libp2p-relay` `v0.9.0`.

- Update to `libp2p-kad` `v0.37.0`.

- Update to `prometheus-client` `v0.16.0`. See [PR 2631].

[PR 2631]: https://github.com/libp2p/rust-libp2p/pull/2631

# 0.5.0

- Update to `libp2p-swarm` `v0.35.0`.

- Update to `libp2p-dcutr` `v0.2.0`.

- Update to `libp2p-ping` `v0.35.0`.

- Update to `libp2p-identify` `v0.35.0`.

- Update to `libp2p-relay` `v0.8.0`.

- Update to `libp2p-kad` `v0.36.0`.

# 0.4.0 [2022-02-22]

- Update to `libp2p-core` `v0.32.0`.

- Update to `libp2p-swarm` `v0.34.0`.

- Update to `libp2p-ping` `v0.34.0`.

- Update to `libp2p-identify` `v0.34.0`.

- Update to `libp2p-relay` `v0.7.0`.

- Update to `libp2p-kad` `v0.35.0`.

- Move from `open-metrics-client` to `prometheus-client` (see [PR 2442]).

- Drop support for gossipsub in wasm32-unknown-unknown target (see [PR 2506]).

[PR 2442]: https://github.com/libp2p/rust-libp2p/pull/2442

[PR 2506]: https://github.com/libp2p/rust-libp2p/pull/2506

# 0.3.0 [2022-01-27]

- Update dependencies.

- Migrate to Rust edition 2021 (see [PR 2339]).

[PR 2339]: https://github.com/libp2p/rust-libp2p/pull/2339

# 0.2.0 [2021-11-16]

- Include gossipsub metrics (see [PR 2316]).

- Update dependencies.

[PR 2316]: https://github.com/libp2p/rust-libp2p/pull/2316

# 0.1.0 [2021-11-01]

- Add initial version.