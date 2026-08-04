
# Changelog

## [v0.10.2] (2026-08-04)

- Fixing `simpleStepTerminable`/`simpleStepTerminableWithInput` hanging the whole engine when the runner fails to dispatch.
- Fixing `stepOnTerminableWithInput` reporting `finished` before its `data` stream had fully drained, which could truncate `out_file`.
- Sequencing GitHub/GitLab status reporting so a state change can no longer race with, and be dropped alongside, the previous one.
- Adding a summary error log when a step's commands exit with a non-zero code.

## [v0.10.1] (2026-05-29)

- No changes in this crate.

## [v0.10.0] (2026-03-02)

- Migrating to new logging system, removing dedicated logging mel file.
- Adding report links for GitHub and GitLab services.
- Exposing running IDs.
- Enabling reporting for distributed execution.

## [v0.9.2] (2026-01-15)

- Fixing `in_*` variable handling in steps.
- Internal cleanup.

## [v0.9.1]

- Adding terminable steps.

## [v0.9.0]

First release.
