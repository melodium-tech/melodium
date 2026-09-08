
# Changelog

## [v0.10.3] (2026-09-08)

- Adding `Value::Packed`, a packed representation for homogeneous scalar arrays (`byte`, numeric, `bool`, `char` vectors), avoiding the per-element `Value` enum overhead a boxed `Vec<Value>` costs (#116).
- Adding panic-free `InputExt`/`OutputExt` (`recv_one_as`, `recv_many_as`, `send_one_as`, `send_many_as`) for casting to/from a concrete type without risking a panic on a type mismatch (#119).
- Adding `Value::estimated_size`, a cheap memory-footprint estimate used to bound transmission buffering.
- Adding `Value::try_data` as a more ergonomic entry point to `GetData`.
- Generating `TransmissionValue`'s per-type conversions instead of hand-duplicating them (#120).

## [v0.10.2] (2026-08-04)

- Adding `World::wait_no_more_tracks` to let continuous tasks detect that no track will ever run again, instead of relying on an arbitrary timeout.
- Fixing multiline string display for values starting with `{` or ending with `}`.

## [v0.10.1] (2026-05-29)

- No changes in this crate.

## [v0.10.0] (2026-03-02)

- Adding debug support to treatment executive.

## [v0.9.2] (2026-01-15)

- No changes in this crate.

## [v0.9.0]

- Adding Deserialize trait.

## [v0.8.0]

- Adding custom data types management.
- Adding generics management.
- Adding traits management.
- Source treatments can have `const` parameters.
- Including attributes.

## [v0.7.2]

- Managing no-data case in transmission.

## [v0.7.0]

- Adding Status for better error management.
- Improving user display of elements.
- Improving LoadingError details.
- Buildable descriptors are able to tell if they depends on identified element.

## [v0.6.1]

- Adding parsing function to Identifier.

## [v0.6.0]

First release.
