
# Changelog

## [Unreleased]

- Adding reading of JSON values (`json/value::|get`, `|index`, `|pointer`, and their treatments), building of objects and arrays (`|object`, `|insert`, `|array`, `|push`, and the `insert`, `array` and `push` treatments), and conversion of JSON values to and from typed values (`|from_json<T: Deserialize>`, `|into_json<T: Serialize>`, `fromJson`, `intoJson`), string maps being objects of strings (#163).
- Fixing `fromNumberU64` and `fromNumberI64`, which narrowed streamed numbers to 16 bits and panicked on larger ones (#163).

## [v0.11.0] (2026-10-15)

- No changes in this crate.

## [v0.10.4] (2026-09-24)

- No changes in this crate.

## [v0.10.3] (2026-09-08)

- Adapting to `melodium-common`'s new panic-free value casting and packed-array APIs (no behavior change).

## [v0.10.2] (2026-08-04)

- No changes in this crate.

## [v0.10.1] (2026-05-29)

- No changes in this crate.

## [v0.10.0] (2026-03-02)

- No changes in this crate.

## [v0.9.2] (2026-01-15)

- No changes in this crate.

## [v0.9.1] (2025-10-23)

- No changes in this crate.

## [v0.9.0]

- Making Json data type deserializable.

## [v0.8.0]

- Adding JSON data type.
- Updating content for new language elements.

## [v0.7.0]

First release.
