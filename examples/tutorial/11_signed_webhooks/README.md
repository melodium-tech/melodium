# 11: Signed Webhooks

**Concepts introduced:** `Secret<string>` parameters, verifying HMAC signatures with `crypto`, checking tokens received as secret headers, gating a value on a verification result.

**Book:** [Secrets](https://doc.melodium.tech/book/en/programming/secrets.html), [Parameters](https://doc.melodium.tech/book/en/programming/parameters.html), [Contexts](https://doc.melodium.tech/book/en/programming/elements/contexts.html).

An HTTP server receiving webhooks from GitHub and GitLab, accepting only the requests proving they come from them. GitHub signs each body with a shared secret and sends the HMAC-SHA256 signature in `X-Hub-Signature-256`; GitLab sends a shared token in `X-Gitlab-Token`. Accepted requests get `200 accepted` and their event name is logged; any other request gets `403 rejected`.

## What it does

```
MELODIUM_SECRET_GITHUB_WEBHOOK_SECRET=gh-shared MELODIUM_SECRET_GITLAB_WEBHOOK_TOKEN=gl-token \
    melodium run Compo.toml
```

In a second terminal:

```
BODY='{"ref":"main"}'
SIGNATURE=$(printf %s "$BODY" | openssl dgst -sha256 -hmac gh-shared | awk '{print $NF}')

curl -X POST http://127.0.0.1:8080/github -H "X-GitHub-Event: push" -H "X-Hub-Signature-256: sha256=$SIGNATURE" -d "$BODY"
# accepted
curl -X POST http://127.0.0.1:8080/github -H "X-GitHub-Event: push" -H "X-Hub-Signature-256: sha256=00ff" -d "$BODY"
# rejected

curl -X POST http://127.0.0.1:8080/gitlab -H "X-Gitlab-Event: Push Hook" -H "X-Gitlab-Token: gl-token" -d "$BODY"
# accepted
curl -X POST http://127.0.0.1:8080/gitlab -H "X-Gitlab-Event: Push Hook" -H "X-Gitlab-Token: wrong" -d "$BODY"
# rejected
```

The server log shows:

```
info: github: push
warning: github: invalid signature, request rejected
info: gitlab: Push Hook
warning: gitlab: invalid token, request rejected
```

To receive real webhooks, the server has to be reachable from GitHub or GitLab (behind a reverse proxy or a tunnel), with the same secret configured in the webhook settings of the repository.

## How it is built

| Element | Kind | Purpose |
|---|---|---|
| `server` | `HttpServer` model | HTTP listener bound to localhost, giving `X-Gitlab-Token` only as a secret |
| `github` | treatment | `POST /github`: verifies the body signature with `crypto/signature/hmac::verify` |
| `gitlab` | treatment | `POST /gitlab`: checks the token with `crypto/token::check` |
| `receivedToken` | treatment | Reads the token from the `secret_headers` of `@HttpRequest` |
| `respond` | treatment | Turns `accepted` or `rejected` into a status, headers and a body |

### Data flow

```
POST /github ─▶ headers ─▶ X-Hub-Signature-256 ─┐
             └▶ body ───────────────────────────┴▶ verify ─▶ valid ───────────▶ 200 accepted, log event
                                                          └▶ invalid/failed ──▶ 403 rejected

POST /gitlab ─▶ @HttpRequest[secret_headers] ─▶ X-Gitlab-Token ─▶ check ─▶ valid ───────────▶ 200 accepted, log event
                                                                         └▶ invalid/failed ──▶ 403 rejected
```

### Reference

- **`crypto`**: [verify](https://doc.melodium.tech/latest/en/crypto/signature/hmac/verify.html), [check](https://doc.melodium.tech/latest/en/crypto/token/check.html)
- **`encoding`**: [encode](https://doc.melodium.tech/latest/en/encoding/encode.html)
- **`http`**: [HttpServer](https://doc.melodium.tech/latest/en/http/server/HttpServer.html), [start](https://doc.melodium.tech/latest/en/http/server/start.html), [connection](https://doc.melodium.tech/latest/en/http/server/connection.html), [@HttpRequest](https://doc.melodium.tech/latest/en/http/server/@HttpRequest.html), [|post](https://doc.melodium.tech/latest/en/http/method/|post.html), [HttpStatus](https://doc.melodium.tech/latest/en/http/status/HttpStatus.html), [|ok](https://doc.melodium.tech/latest/en/http/status/|ok.html), [|forbidden](https://doc.melodium.tech/latest/en/http/status/|forbidden.html)
- **`net`**: [|localhost_ipv4](https://doc.melodium.tech/latest/en/net/ip/|localhost_ipv4.html), [|from_ipv4](https://doc.melodium.tech/latest/en/net/ip/|from_ipv4.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfo](https://doc.melodium.tech/latest/en/std/engine/log/logInfo.html), [logWarning](https://doc.melodium.tech/latest/en/std/engine/log/logWarning.html), [logError](https://doc.melodium.tech/latest/en/std/engine/log/logError.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html), [one](https://doc.melodium.tech/latest/en/std/flow/one.html), [check](https://doc.melodium.tech/latest/en/std/flow/check.html), [stream](https://doc.melodium.tech/latest/en/std/flow/stream.html), [releaseBlock](https://doc.melodium.tech/latest/en/std/flow/releaseBlock.html), [|get](https://doc.melodium.tech/latest/en/std/data/map/|get.html), [StringMap](https://doc.melodium.tech/latest/en/std/data/string_map/StringMap.html), [|map](https://doc.melodium.tech/latest/en/std/data/string_map/|map.html), [get](https://doc.melodium.tech/latest/en/std/data/string_map/block/get.html), [unwrapOr](https://doc.melodium.tech/latest/en/std/ops/option/block/unwrapOr.html)

## Runtime behaviour

1. `verify` computes the HMAC of the body as it streams in, and compares it in constant time with the signature once the body ends. A missing header, a header without the `sha256=` prefix, or a signature that is not valid hexadecimal all count as `invalid`.
2. The status is only decided once the verification ends, so no request is ever answered `200` before its signature is known to be good.
3. The event name is read from `X-GitHub-Event` or `X-Gitlab-Event` as soon as the request arrives, but `releaseBlock` holds it until `valid` is emitted: the log line only exists for accepted requests. When the request is rejected, `valid` never comes and the held value is dropped.
4. `invalid` and `failed` both lead to `403`, merged by `one<void>()`. `failed` means the secret itself could not be revealed (such as an unset environment variable), and its reason goes to the error log, never to the client.

### Key Mélodium patterns used

- **Shared values as secrets**: `github_secret` and `gitlab_token` are `Secret<string>` parameters defaulting to locators, `"env:MELODIUM_SECRET_GITHUB_WEBHOOK_SECRET"` and `"env:MELODIUM_SECRET_GITLAB_WEBHOOK_TOKEN"`. Programs only get the environment variables whose name starts with `MELODIUM_SECRET_`, and the values are revealed only inside `verify` and `check`, so they appear in no parameter, log or report. Another locator can be given instead, such as `--github_secret file:/run/secrets/github_webhook`.
- **Secret headers**: `HttpServer`'s `secret_headers` lists headers given only as secrets, in `@HttpRequest[secret_headers]` instead of the plain request headers. A token received this way cannot be logged or sent back by mistake; `crypto/token::check` compares it without exposing it. `Authorization` and `Cookie` are always handled this way.
- **One response treatment for every route**: `respond` takes two `Block<void>` outcomes and drives the three outputs `connection` needs (`status`, `headers`, `data`), with `one<T>()` merging the two possible values of each.
- **Gating with `releaseBlock`**: a value known early is released only when a later signal arrives, here the event name and the verification result.

Back to the [examples index](../../README.md) for the showcase track, which builds on this to distribute AI workloads and CI/CD pipelines.
