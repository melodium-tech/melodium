
# Changelog

## [v0.10.3] (2026-09-08)

- Adding chunked, size-capped wire framing: a hard ceiling on a single frame (previously trusting a peer's claimed length up to ~4 GiB) and a soft target for splitting large batches, both overridable (`MELODIUM_DIST_PROTOCOL_MAX_FRAME_BYTES`, `MELODIUM_DIST_PROTOCOL_MAX_BATCH_CHUNK_BYTES`) (#114).
- Bounding log/debug event channels and capping in-flight concurrent message handling per connection, so a fast peer or a slow local consumer now applies back-pressure instead of growing memory without limit (`MELODIUM_DIST_LOG_CHANNEL_CAPACITY`, `MELODIUM_DIST_DEBUG_CHANNEL_CAPACITY`, `MELODIUM_DIST_MAX_CONCURRENT_MESSAGES`) (#115).
- Fixing a race where a `CloseInput`/`CloseOutput` message could be handled before a same-port data write still in flight, silently dropping the tail of a stream.

## [v0.10.2] (2026-08-04)

- Fixing protocol idle timeout being too close to the keepalive probe interval, causing healthy connections to be torn down under load.
- Adding a teardown grace period safety net (overridable via `MELODIUM_DIST_TEARDOWN_TIMEOUT_SECS`) so a stuck connection/log/debug teardown cannot hang a run forever.
- Making `launch_listen`, `launch_listen_localcert`, and `launch_listen_unsecure` report whether the run actually launched.

## [v0.10.1] (2026-05-29)

- No changes in this crate.

## [v0.10.0] (2026-03-02)

- Adding debug data transmission.
- Adding launch and end signals.
- Updating raw data transmission.
- Updating distribution protocol.
- Enabling execution reporting.

## [v0.9.0]

First release.
