
# Changelog

## [v0.11.0] (unreleased)

- Supporting `Secret<T>` in elements: elements declaring `secrets_access` get it, as `secrets_access` in treatments and `secrets_access()` in models, models register secret sources in their `secret_sources` function, and contexts can have secret fields (#134, #136).
- Giving treatment bodies the `world` they run in, as a weak reference; `world`, `track_id` and `secrets_access` are reserved names in treatments (#134).
- Allowing `none` defaults for `Option<T>` parameters of Rust-declared elements (#123).
- Breaking: `invoke_source` is declared in `#[mel_model]` with `invoke_source method_name` by the models needing it, like `initialize` and `shutdown`; other models no longer define it, and a model relying on it without declaring it is no longer called (#132).

## [v0.10.4] (2026-09-24)

- No changes in this crate.

## [v0.10.3] (2026-09-08)

- Generating code that delegates to `Value`'s own `Vec`/`Option` conversions (including packed-array auto-packing/unpacking) for scalar and generic type chains, instead of hand-rolling a per-element match (#116, #120).

## [v0.10.2] (2026-08-04)

- No changes in this crate.

## [v0.10.1] (2026-05-29)

- No changes in this crate.

## [v0.10.0] (2026-03-02)

- Updating macros for debug system support.

## [v0.9.2] (2026-01-15)

- No changes in this crate.

## [v0.9.0]

- Improvement of macros usage.

## [v0.8.0]

- Adding custom data types management.
- Adding generics management.
- Adding traits management.
- Source treatments can have `const` parameters.
- Including language attributes.

## [v0.7.0]

- Fixing boolean default parameter parsing issue.

## [v0.6.0]

First release.
