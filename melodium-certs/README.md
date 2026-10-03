
# Mélodium certs crate

Mélodium root certificates.

This crate provides root certificates for the Mélodium environment.

Look at the [Mélodium Project](https://melodium.tech/) for more detailed information.

## Renewing certificates

Distribution clients trust every certificate of `melodium-ca.pem`. Nodes present
`melodium-distribution/melodium-chain.pem` (localhost leaf, then intermediate) with
`melodium-distribution/melodium-localhost.key.pem` in `--localhost` mode, or the chain
given with `--certificate` and `--key`.

To renew them before they expire:
1. Issue a new root, a new intermediate signed by it, and a new localhost leaf
   (`127.0.0.1` and `::1`) signed by the intermediate.
2. Add the new root after the current one in `melodium-ca.pem`, so that clients trust
   both, and replace the chain and localhost key of `melodium-distribution`.
3. Re-issue the certificates of remote nodes from the new intermediate.
4. Once every node uses the new certificates, at least one release later, remove the
   previous root from `melodium-ca.pem`.

The `certificates_expiry` test of `melodium-distribution` fails when an embedded
certificate expires within 90 days.



