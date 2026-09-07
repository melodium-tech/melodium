# 05: HTTP Client

**Concepts introduced:** calling a remote HTTP API (`http/client`), telling a network failure apart from an application-level error.

**Book:** [Error Handling](https://doc.melodium.tech/book/en/programming/error_handling.html).

Fetches one post from a public test API ([jsonplaceholder.typicode.com](https://jsonplaceholder.typicode.com)), re-serialises its JSON body, and writes it to a file.

## What it does

```
melodium run Compo.toml --post_id 1
```

- Builds the request URL from `post_id` with `|format`.
- Logs when the request completes, and separately handles a *technical* failure (DNS, connection, timeout: `fetch.failed`/`fetch.error`) versus a body that fails to parse.
- Writes the parsed-and-reserialised JSON body to `post.txt`.

## How it is built

No models: `http/client/util::get` needs no client model for a one-off request (see [06_http_server_api](../06_http_server_api/) for the model-based server side, and later examples for `HttpClient` as a model when a connection should be reused).

### Data flow

```
get ──▶ decode ──▶ toJson ──▶ unwrapOr ──▶ toString ──▶ writeTextLocal
  │
  ├──▶ completed ──▶ log "request completed"
  ├──▶ failed    ──▶ log "request failed technically"
  └──▶ error     ──▶ log error message
```

### Reference

- **`encoding`**: [decode](https://doc.melodium.tech/latest/en/encoding/decode.html)
- **`fs`**: [writeTextLocal](https://doc.melodium.tech/latest/en/fs/local/writeTextLocal.html)
- **`http`**: [get](https://doc.melodium.tech/latest/en/http/client/util/get.html)
- **`json`**: [Json](https://doc.melodium.tech/latest/en/json/Json.html), [toJson](https://doc.melodium.tech/latest/en/json/toJson.html), [|null](https://doc.melodium.tech/latest/en/json/value/|null.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html), [logError](https://doc.melodium.tech/latest/en/std/engine/log/logError.html), [logErrorMessage](https://doc.melodium.tech/latest/en/std/engine/log/logErrorMessage.html), [|format](https://doc.melodium.tech/latest/en/std/text/compose/|format.html), [|entry](https://doc.melodium.tech/latest/en/std/data/string_map/|entry.html), [|map](https://doc.melodium.tech/latest/en/std/data/string_map/|map.html), [toString](https://doc.melodium.tech/latest/en/std/conv/toString.html), [unwrapOr](https://doc.melodium.tech/latest/en/std/ops/option/unwrapOr.html)

## Runtime behaviour

1. `get(url=...)` fires on `startup.trigger` and streams the response body through `data`, independently of `status`/`completed`/`failed`/`error`, which fire once each.
2. `decode` turns the raw byte body into a `Stream<string>`, parsed with `toJson`, unwrapped, and re-serialised: this round-trip is a good way to confirm a response really is valid JSON without changing its meaning.

### Key Mélodium patterns used

- **Technical failure vs. data failure**: `fetch.failed`/`fetch.error` fire when the *request itself* could not be completed; a response that arrives successfully but contains invalid JSON is a completely separate, later failure mode (`toJson`'s `Option` coming back `none`). Do not conflate the two.
- **`|format` for URLs**: the same function used for the greeting in example 01 works just as well to build a URL from a parameter.

Next: [06_http_server_api](../06_http_server_api/) introduces building an HTTP server and the `@HttpRequest` context.
