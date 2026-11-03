
# Changelog

## [Unreleased]

- Adding `std/flow/scatter`: `scatter` makes one track per value of a stream, at most `parallel` at once, where the generic `scattered` source gives the value and a value shared by all, the tracks inheriting the contexts of the track of `scatter` with `@Scattered`, and `gather` tells how each ended, a track ending without telling being counted as failed; `scatterGathering` also gives the values the tracks give to `gatherValue`, in the order they end (#168).
- Adding `std/secret::scan`, giving data further unless it holds a value the engine revealed, holding back the bytes that could start one, and naming the secret when one is found (#162).

## [v0.11.0] (2026-10-15)

- Using async-channel and async-lock instead of async-std, which is discontinued (#104).
- Adding `std/secret`: secrets from the environment, files and locators (`|from_environment`, `|from_file`, `|locate`), policy narrowing, derivations (`|format`, `|base64`, `|url_encode`, `|to_bytes`), the `conceal` and `reveal` treatments, and `std/secret/block`, deriving and narrowing secrets received at runtime; secrets from the environment and from files are never plainly revealed (#134, #135).

## [v0.10.4] (2026-09-24)

- No changes in this crate.

## [v0.10.3] (2026-09-08)

- Adapting to `melodium-common`'s new panic-free value casting and packed-array APIs (no behavior change).

## [v0.10.2] (2026-08-04)

- No changes in this crate.

## [v0.10.1] (2026-05-29)

- No changes in this crate.

## [v0.10.0] (2026-03-02)

- Adding engine module exposing running IDs.
- Migrating logging management.

## [v0.9.2] (2026-01-15)

- Fixing flow status notification.

## [v0.9.1]

- Adding `flow/closeBlock`, `ops/block/equal`, and `ops/block/equalTo` treatments.

## [v0.9.0]

- Moving Map data type.
- Creating StringMap data type.
- Creating new flow management treatments.
- Creating Concentrator model.
- Providing more operations.

## [v0.8.0]

- First release.
- Reuniting disparate fundamental libraries for Mélodium usage.
