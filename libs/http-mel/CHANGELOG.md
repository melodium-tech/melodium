
# Changelog

## [v0.11.0] (unreleased)

- Adding `secret_headers` to `HttpClient` and to the client helpers, revealed for each request, and `requestWithSecretHeaders` and `requestWithBodyAndSecretHeaders` for secret headers received at runtime (#136).
- Fixing `HttpClient` ignoring its `headers` parameter.
- Breaking: the `Authorization` and `Cookie` headers of incoming requests are only given as `@HttpRequest` secrets, out of the request headers, as are the headers listed in the new `secret_headers` parameter of `HttpServer`, in the `secret_headers` map of `@HttpRequest` (#136).

## [v0.10.4] (2026-09-24)

- No changes in this crate.

## [v0.10.3] (2026-09-08)

- Documenting that `http_server`'s `status`/`headers`/`data` outputs must be driven from `started`, not from a trigger derived from `data`, since a request with no body never emits on `data`.
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

- Make use of StringMap instead of Map for headers.

## [v0.8.0]

- Refactoring implementation.
- Updating content for new language elements.

## [v0.7.0]

First release.
