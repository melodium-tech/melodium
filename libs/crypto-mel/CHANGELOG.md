# Changelog

## [v0.11.0] (unreleased)

- First version: HMAC signing and verification of data (`crypto/signature/hmac::sign`, `crypto/signature/hmac::verify`) with SHA-256, SHA-384, SHA-512 and SHA-1, and constant-time checking of received tokens against secrets (`crypto/token::check`), such as for webhooks (#136).
- Verifying Ed25519 signatures of data (`crypto/signature/ed25519::verify`), such as Discord interactions (#136).
- Signing JSON Web Tokens with HMAC, RSA, ECDSA or Ed25519, given as secrets (`crypto/token/jwt::sign`), such as to authenticate as a GitHub App, and verifying received ones against a shared secret or a public key (`crypto/token/jwt::verify`) (#136).
- Generating random tokens as secrets (`crypto/random::token`) (#136).
- Encoding signatures and tokens in hexadecimal, base64 or base64url (#136).
- Built on the RustCrypto crates (`hmac`, `sha1`, `sha2`, `p256`, `p384`, `ed25519-dalek`, `rsa`); `rsa` 0.10, matching the others, is only available as a release candidate yet, pinned to `0.10.0-rc.19` (#136).
