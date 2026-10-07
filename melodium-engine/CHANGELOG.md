
# Changelog

## [v0.11.0] (unreleased)

- Adding the `env:` and `file:` secret sources, the first one giving only the environment variables prefixed `MELODIUM_SECRET_`, behind the `environment` and `filesystem` features and never plainly revealing their secrets, the secret audit as debug events (and log lines for refusals and failures), and the masking of revealed values in logs (#134, #137).
- Adding `Engine::check_secrets` and `Engine::secret_access`, breaking for `Engine` implementors (#134, #139).
- Ending runs gracefully on SIGINT, SIGTERM and SIGHUP, and as a best effort on the console control events of Windows, through `interruption::live_until_interrupted` (#131).
- Fixing models keeping their world alive through a reference cycle: models hold it weakly, and `world()` gives an owned `Arc` (#132).
- Wiping the masking registry once the last log is delivered, and before exit (#140).

## [v0.10.4] (2026-09-24)

- No changes in this crate.

## [v0.10.3] (2026-09-08)

- Adding byte-size watermark flow control to transmission `Output`, blocking the producer only once buffered data crosses a threshold instead of on every full receiver slot (overridable via `MELODIUM_TRANSMISSION_MAX_BUFFERED_BYTES`) (#112).
- Sharing one `Arc`-wrapped batch across fan-out receivers instead of deep-cloning it per receiver (#113).

## [v0.10.2] (2026-08-04)

- Implementing `World::wait_no_more_tracks`, fixing continuous tasks hanging forever when waiting on an event tied to a track that never runs.
- Waiting for log/debug transmission tasks to fully drain before `live()` returns, so listeners can rely on it as "everything has been delivered".

## [v0.10.1] (2026-05-29)

- No changes in this crate.

## [v0.10.0] (2026-03-02)

- Adding debug system with data dump capability (#102).
- Adding execution group ID and renaming job identifier to run ID.
- Migrating logging management to engine.
- Improving data transmission.
- Exposing running track count.

## [v0.9.2] (2026-01-15)

- Fixing I/O behavior for Block transmission.

## [v0.9.1]

- Improving track building mechanism.
- Adding stack size increase on need.

## [v0.9.0]

- Improving data transmission.
- Making engine able to manage different launch modes.

## [v0.8.0]

- Adding custom data types management.
- Adding generics management.
- Adding traits management.
- Source treatments can have `const` parameters.
- Including attributes.

## [v0.7.2]

- Fixing parameters const/var and contexts transmission.
- Checking empty data transmission between treatments.

## [v0.7.1]

- Improving shutdown mechanism.

## [v0.7.0]

- Improving design error reporting.
- Refactoring many design internal procedures.
- Makes collection updatable in designers.

## [v0.6.0]

First release.
