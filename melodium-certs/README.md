
# Mélodium certs crate

Mélodium root certificates.

This crate provides root certificates for the Mélodium environment.

Look at the [Mélodium Project](https://melodium.tech/) for more detailed information.

## Renewing certificates

Distribution clients trust every certificate of `melodium-ca.pem`. A node presents a
single chain: `melodium-distribution/melodium-chain.pem` (localhost leaf, then
intermediate) with `melodium-distribution/melodium-localhost.key.pem` in `--localhost`
mode, or the chain given with `--certificate` and `--key`. Nodes launched by Mélodium
services get certificates issued by the intermediate of the services, also signed by
the root.

A client only accepts a chain leading to a root it trusts, so clients have to trust the
new root before nodes present chains leading to it:
1. Issue a new root, a new intermediate signed by it, and a new localhost leaf
   (`127.0.0.1` and `::1`) signed by this intermediate. Issue a new intermediate for
   Mélodium services from the new root.
2. Release with the new root added after the current one in `melodium-ca.pem`, keeping
   the current chain and localhost key. This release trusts both roots, and still
   connects with previous releases.
3. In a later release, replace the chain and localhost key of `melodium-distribution`,
   and move Mélodium services to their new intermediate. Releases from step 2 on accept
   the new chains, earlier ones no longer connect to nodes presenting them.
4. Once no supported release relies on the previous root, or once it has expired,
   remove it from `melodium-ca.pem`.

Steps 2 and 3 both have to happen before the current chain expires. When the engines
involved always run the same release, as with `--localhost` within one deployment,
steps 2 and 3 can be the same release.

The `certificates_expiry` test of `melodium-distribution` fails when an embedded
certificate expires within 90 days.
