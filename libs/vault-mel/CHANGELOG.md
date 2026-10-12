# Changelog

## [v0.11.0] (unreleased)

- First version: `Vault` model, registering a secret source for HashiCorp Vault and OpenBao key/value secrets, and `get` treatment, giving secrets from paths known at runtime with the policy it is given; its credentials are read by default from the usual Vault variables prefixed `MELODIUM_SECRET_`, and two models with the same `source` make the launch fail.
