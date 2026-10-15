
# Changelog

## [v0.11.0] (unreleased)

- Calling remote providers in a Tokio context given by async-compat, and loading models on the threads of the blocking crate, instead of async-std, which is discontinued (#104).
- Breaking: taking the `api_key` of `RemoteLlm`, `RemoteStt` and `RemoteTts`, and the `token` of `HfHub`, as secrets (`Option<Secret<string>>`), revealed only to build the provider or the Hub client. Remote providers are built on first use instead of when the program starts, as secrets are only revealed once every model is initialized, and a failed build is given on `error` instead of "provider not initialized".

## [v0.10.4] (2026-09-24)

- No changes in this crate.

## [v0.10.3] (2026-09-08)

- Updating `candle-core`, `candle-nn`, and `candle-transformers` to 0.11.
- Adapting to `melodium-common`'s new panic-free value casting and packed-array APIs (no behavior change).

## [v0.10.2] (2026-08-04)

- No changes in this crate.

## [v0.10.1] (2026-05-29)

First release.

- Adding local ML model support (Mistral LLM, Whisper STT).
- Adding remote LLM, STT, and TTS API treatments.
- Adding HuggingFace repository integration.
- Enabling WebAssembly compilation support.
