# 06: HTTP Server API

**Concepts introduced:** the `HttpServer` model, routing with `connection`, the `@HttpRequest` context.

**Book:** [Models](https://doc.melodium.tech/book/en/programming/elements/models.html), [Contexts](https://doc.melodium.tech/book/en/programming/elements/contexts.html), [Tracks](https://doc.melodium.tech/book/en/programming/concepts/tracks.html) (the HTTP server example there is essentially this one).

A small HTTP server with three routes: a fixed status endpoint, one that reads request metadata from the `@HttpRequest` context, and one that parses a JSON body and replies with a JSON object built from it.

## What it does

```
melodium run Compo.toml --port 8080

curl http://127.0.0.1:8080/health
curl http://127.0.0.1:8080/whoami
curl -X POST http://127.0.0.1:8080/greet -d '{"name":"Ada"}'
```

```json
{"status":"ok"}
{"path":"/whoami","route":"/whoami"}
{"message":"thanks for the greeting!","received":"{\"name\":\"Ada\"}"}
```

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `server` | `HttpServer` | HTTP listener bound to localhost |

`start[http_server=server]()` binds the socket once at startup; `connection[http_server=server](method=..., route=...)` is instantiated once per route and creates a new track, with the `@HttpRequest` context available, for every matching incoming request.

### Data flow (per route)

```
connection.started ──▶ status/headers ──▶ connection.status/headers
connection.started ──▶ build response body ──▶ connection.data
connection.data (incoming) ──▶ [only for /greet: parse body] ──▶ connection.data (outgoing)
```

### Reference

- **`encoding`**: [decode](https://doc.melodium.tech/latest/en/encoding/decode.html), [encode](https://doc.melodium.tech/latest/en/encoding/encode.html)
- **`http`**: [HttpServer](https://doc.melodium.tech/latest/en/http/server/HttpServer.html), [start](https://doc.melodium.tech/latest/en/http/server/start.html), [connection](https://doc.melodium.tech/latest/en/http/server/connection.html), [@HttpRequest](https://doc.melodium.tech/latest/en/http/server/@HttpRequest.html), [|get](https://doc.melodium.tech/latest/en/http/method/|get.html), [|post](https://doc.melodium.tech/latest/en/http/method/|post.html), [|ok](https://doc.melodium.tech/latest/en/http/status/|ok.html), [HttpStatus](https://doc.melodium.tech/latest/en/http/status/HttpStatus.html)
- **`json`**: [Json](https://doc.melodium.tech/latest/en/json/Json.html), [toJson](https://doc.melodium.tech/latest/en/json/toJson.html), [|null](https://doc.melodium.tech/latest/en/json/value/|null.html), [fromStringMap](https://doc.melodium.tech/latest/en/json/value/fromStringMap.html)
- **`net`**: [|localhost_ipv4](https://doc.melodium.tech/latest/en/net/ip/|localhost_ipv4.html), [|from_ipv4](https://doc.melodium.tech/latest/en/net/ip/|from_ipv4.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html), [stream](https://doc.melodium.tech/latest/en/std/flow/stream.html), [fill](https://doc.melodium.tech/latest/en/std/flow/fill.html), [StringMap](https://doc.melodium.tech/latest/en/std/data/string_map/StringMap.html), [|map](https://doc.melodium.tech/latest/en/std/data/string_map/|map.html), [|entry](https://doc.melodium.tech/latest/en/std/data/string_map/|entry.html), [|insert](https://doc.melodium.tech/latest/en/std/data/string_map/|insert.html), [entry](https://doc.melodium.tech/latest/en/std/data/string_map/entry.html), [insert](https://doc.melodium.tech/latest/en/std/data/string_map/insert.html), [unwrapOr](https://doc.melodium.tech/latest/en/std/ops/option/unwrapOr.html), [toString](https://doc.melodium.tech/latest/en/std/conv/toString.html), [toVoid](https://doc.melodium.tech/latest/en/std/conv/toVoid.html)

## Runtime behaviour

1. **Drive a route's response from `connection.started`**, not from a trigger derived from `connection.data` (the incoming body). `connection.started` is a `Block<void>` that fires as soon as the connection is accepted, regardless of whether the request has a body. A GET request has no body, so a stream never starts on `connection.data`, and anything gated on "first byte of the body" simply never fires, leaving the server hanging on that route. `/greet` (POST, with an actual body) would work either way, which is exactly the trap: a body-derived trigger looks correct until tested against a route with no body.
2. `/whoami` reads `@HttpRequest[route]` and `@HttpRequest[path]` directly as values inside `describe`, a treatment that `require`s the context: it can only be instantiated inside a track that provides `@HttpRequest`, which `connection` guarantees.
3. `/greet` parses the JSON body and rebuilds a response with `std/data/string_map::entry`/`insert` + `fromStringMap`. There is no field-by-field access into a parsed `Json` value in the `json` package itself: reaching into `{"name": "Ada"}` to pull out `"Ada"` needs the JavaScript engine ([08_javascript_transform](../08_javascript_transform/)); here the whole body is echoed back as one JSON string value instead.

### Key Mélodium patterns used

- **`connection.started` vs. body-derived triggers**: see above; this is the one thing to get right in every HTTP server route in this codebase.
- **`require @HttpRequest`**: a treatment that needs context data declares it with `require`; Mélodium then only allows it to be used where that context is actually provided.
- **One route, one treatment**: `health`, `whoami`, and `greet` are independent treatments, each owning its own `connection` instance; `main` just instantiates all three against the same `server` model.

Next: [07_sql_crud_api](../07_sql_crud_api/) adds a SQL-backed model to persist data across requests.
