# 05: HTTP Client

**Concepts introduced:** calling a remote HTTP API (`http/client`), telling a network failure apart from an application-level error, a real streaming gotcha.

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
get ──▶ decode ──▶ drop trailing blank chunk ──▶ toJson ──▶ unwrapOr ──▶ toString ──▶ writeTextLocal
  │
  ├──▶ completed ──▶ log "request completed"
  ├──▶ failed    ──▶ log "request failed technically"
  └──▶ error     ──▶ log error message
```

## Runtime behaviour

1. `get(url=...)` fires on `startup.trigger` and streams the response body through `data`, independently of `status`/`completed`/`failed`/`error`, which fire once each.
2. **A verified gotcha:** `decode` does not always emit the whole body as a single string. Running this exact request showed it emits the full JSON as one item, *then a trailing empty string* when the underlying stream closes. Feeding that directly into `toJson` would parse the empty string too, producing a spurious extra `null` after the real value: confirmed by removing the blank-filter below and inspecting `post.txt`. The fix is the same "drop blanks" idiom as examples 03/04: `exact(pattern="") + not + filter`, keeping only non-blank chunks.
3. The (now single-chunk) body is parsed with `toJson`, unwrapped, and re-serialised: this round-trip is a good way to confirm a response really is valid JSON without changing its meaning.

### Key Mélodium patterns used

- **Technical failure vs. data failure**: `fetch.failed`/`fetch.error` fire when the *request itself* could not be completed; a response that arrives successfully but contains invalid JSON is a completely separate, later failure mode (`toJson`'s `Option` coming back `none`). Do not conflate the two.
- **Never assume "one send = one stream item"**: the trailing empty chunk above is a concrete example of why it matters; verify with `melodium run`, not just `melodium check` (which only checks types, not values or cardinality).
- **`|format` for URLs**: the same function used for the greeting in example 01 works just as well to build a URL from a parameter.

Next: [06_http_server_api](../06_http_server_api/) introduces building an HTTP server and the `@HttpRequest` context.
