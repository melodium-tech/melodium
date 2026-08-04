
# Changelog

## [v0.10.2] (2026-08-04)

- Fixing `DistributionEngine::stop`/`continuous` hanging forever when `start` is never called (e.g. worker dispatch failing upstream).
- Retrying keepalive `Probe` up to 3 times before treating a connection as dead, instead of failing it on a single transient send error.

## [v0.10.1] (2026-05-29)

- No changes in this crate.

## [v0.10.0] (2026-03-02)

- Adding debug data transmission.
- Exposing running IDs.

## [v0.9.1]

- Improving track distribution synchronisation mechanism.

## [v0.9.0]

First release.

