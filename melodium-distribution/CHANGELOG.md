
# Changelog

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
