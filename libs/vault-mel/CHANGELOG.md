# Changelog

## [v0.11.0] (unreleased)

- First version: `Vault` model, registering a secret source for HashiCorp Vault and OpenBao key/value secrets, and `get` treatment, its credentials read by default from the usual Vault variables prefixed `MELODIUM_SECRET_`; two models with the same `source` make the launch fail.
