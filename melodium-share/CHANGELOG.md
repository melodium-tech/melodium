
# Changelog

## [v0.10.3] (2026-09-08)

- Adding a wire-serializable `TransmissionValue`, keeping a whole stream batch as one CBOR-encoded unit instead of re-encoding it tick by tick (#114).
- Exposing `ContextualEnvironment`, `DataContent`, `EventKind`, `HostTreatment`, `InfoTrack`, `TrackCreation`, `TrackResult`, and `TransmissionDetails`, and making `ContextualEnvironment`'s fields public.

## [v0.10.2] (2026-08-04)

- No changes in this crate.

## [v0.10.1] (2026-05-29)

- No changes in this crate.

## [v0.10.0] (2026-03-02)

- Adding debug data structures.
- Adding execution reporting structures.
- Enabling reporting for distributed execution.

## [v0.9.0]

First release.
