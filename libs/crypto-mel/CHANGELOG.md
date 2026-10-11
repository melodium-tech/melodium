# Changelog

## [v0.11.0] (unreleased)

- First version: HMAC signing and verification of data (`crypto/hmac::sign`, `crypto/hmac::verify`) with SHA-256, SHA-384, SHA-512 and SHA-1, and constant-time checking of received tokens against secrets (`crypto/token::check`), such as for webhooks (#136).
- Verifying Ed25519 signatures of data (`crypto/ed25519::verify`), such as Discord interactions (#136).
- Signing JSON Web Tokens with HMAC, RSA, ECDSA or Ed25519, as secrets made when revealed (`crypto/jwt::|sign`), such as to authenticate as a GitHub App, and verifying received ones against a public key (`crypto/jwt::verify`) (#136).
- Generating random tokens as secrets (`crypto/random::token`) (#136).
- Encoding signatures and tokens in hexadecimal, base64 or base64url (#136).
