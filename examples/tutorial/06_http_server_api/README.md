# 06: HTTP Server API

**Concepts introduced:** the `HttpServer` model, routing with `connection`, the `@HttpRequest` context, a verified server-side gotcha.

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

## Runtime behaviour

1. **A verified gotcha, and why it matters:** the response for a route must be driven by `connection.started` (`Block<void>`, fires as soon as the connection is accepted), *not* by a trigger derived from `connection.data` (the incoming body). A GET request has no body, so a stream never starts on `connection.data`, and anything gated on "first byte of the body" simply never fires; the server hangs forever on that route. This was confirmed the hard way: an earlier version of `/health` and `/whoami` in this exact file returned nothing at all over `curl` until the trigger was switched to `connection.started`. `/greet` (POST, with an actual body) worked by coincidence either way, which is exactly the trap: it looks correct until tested against a route with no body.
2. `/whoami` reads `@HttpRequest[route]` and `@HttpRequest[path]` directly as values inside `describe`, a treatment that `require`s the context: it can only be instantiated inside a track that provides `@HttpRequest`, which `connection` guarantees.
3. `/greet` parses the JSON body and rebuilds a response with `std/data/string_map::entry`/`insert` + `fromStringMap`. There is no field-by-field access into a parsed `Json` value in the `json` package itself: reaching into `{"name": "Ada"}` to pull out `"Ada"` needs the JavaScript engine (example 08); here the whole body is echoed back as one JSON string value instead.

### Key Mélodium patterns used

- **`connection.started` vs. body-derived triggers**: see above; this is the one thing to get right in every HTTP server route in this codebase.
- **`require @HttpRequest`**: a treatment that needs context data declares it with `require`; Mélodium then only allows it to be used where that context is actually provided.
- **One route, one treatment**: `health`, `whoami`, and `greet` are independent treatments, each owning its own `connection` instance; `main` just instantiates all three against the same `server` model.

Next: [07_sql_crud_api](../07_sql_crud_api/) adds a SQL-backed model to persist data across requests.
