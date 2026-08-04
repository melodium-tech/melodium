
# Changelog

## [v0.10.2] (2026-08-04)

- Adding a detection timeout (overridable via `MELODIUM_COMPOSE_DETECTION_TIMEOUT_SECS`) so an unresponsive `podman`/`docker` daemon can no longer block `compose()`, and the whole distributed run, forever.
- Retrying the same batch instead of dropping it and advancing when sending logs or debug events to S3 fails.

## [v0.10.1] (2026-05-29)

- Adding program details to execution reporting (#106).

## [v0.10.0] (2026-03-02)

- Adding execution reporting (launch/end signals, chunk and data transmission).
- Adding execution group ID and run ID.
- Enabling WebAssembly compilation.
- Adding child process timeout.
- Turning connection timeout hard errors into soft ones.
- Suppressing pull progress output.

## [v0.9.2] (2026-01-15)

- Fixing SpawnTerminable behavior (#103).
- Adding network alias support in compose.
- Adding user definition in compose.

## [v0.9.0]

First release.

