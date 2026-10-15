
# Changelog

## [v0.11.0] (unreleased)

- Breaking: renewing the root certificate, "Mélodium Distribution Root G2", valid until 2036; the previous root, expiring on 2026-12-23, is no longer trusted, so certificates issued under it are refused (#128).
- Breaking: `ROOT_CERTIFICATE` is a `&[u8]`, so that the file can hold several roots, such as the new one next to the current one during a renewal (#128).

## [v0.10.4] (2026-09-24)

- No changes in this crate.

## [v0.10.3] (2026-09-08)

- No changes in this crate.

## [v0.10.2] (2026-08-04)

- No changes in this crate.

## [v0.10.1] (2026-05-29)

- No changes in this crate.

## [v0.10.0] (2026-03-02)

- No changes in this crate.

## [v0.9.2] (2026-01-15)

- No changes in this crate.

## [v0.9.0]

First release.
