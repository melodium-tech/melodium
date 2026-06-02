# SQL User API

A minimal REST API backed by SQLite (or PostgreSQL) that demonstrates HTTP server integration with a SQL connection pool in Mélodium. The database URL is the only required parameter; the schema is created automatically on first run.

## What it does

- Creates a `users` table if it does not exist.
- Starts an HTTP server on the configured port (default 8080).
- **`GET /users`** queries all rows from the `users` table and logs them as structured info entries; responds with `"OK\n"`.
- **`POST /users`** decodes the request body as JSON, converts it to a string, and echoes it back as the response body.

```
melodium run Compo.toml -- --db_url sqlite://users.db

# list users
curl http://127.0.0.1:8080/users

# echo JSON body
curl -X POST http://127.0.0.1:8080/users \
     -H "Content-Type: application/json" \
     -d '{"name":"Alice","email":"alice@example.com"}'
```

## How it is built

### Models

| Model | Type | Purpose |
|---|---|---|
| `AppDb` | `SqlPool` | Connection pool; `url` from `db_url` param, 1 to 5 connections |
| `server` | `HttpServer` | HTTP listener on localhost at chosen port |

### Treatments

**`main`** is the entry point. It sequences startup -> connect -> createTable -> HTTP start, then wires the two route handlers.

**`createTable`** runs `CREATE TABLE IF NOT EXISTS users ...` via `executeRaw`. It converts the `affected: Block<u64>` count to a `Block<void>` trigger using `check<u64>()`, then emits `done`.

**`listUsers`** handles `GET /users`. It calls `queryUsers` for each request, pipes rows into `logDataInfos<Map>`, and sends a fixed `"OK\n"` string back encoded as bytes.

**`queryUsers`** runs `SELECT id, name, email FROM users ORDER BY id`. It emits an empty `Map` binding, passes it to `sqlFetch`, and streams resulting `Map` rows as output.

**`echoCreate`** handles `POST /users`. It decodes request bytes, parses JSON, converts to string, and encodes back to bytes.

### Data flow

```
startup
  -> connect[db]
  -> createTable[db]
       -> executeRaw (CREATE TABLE)
       -> check<u64>  (discard u64, keep void trigger)
       -> emit<void>
  -> start[server]   (HTTP up)
  -> listUsers[db, server]
  -> echoCreate[server]
```

**`GET /users` per-request track:**
```
connection.data -> bodyTrigger -> status 200 + headers
               -> queryUsers -> logDataInfos<Map>
               -> emit "OK\n" -> stream<string> -> encode -> response
```

**`POST /users` per-request track:**
```
connection.data -> decode -> toJson -> unwrapOr -> toString<Json> -> encode -> response
```

## Runtime behaviour

1. `startup` fires; `connect[db]` opens the connection pool to the database URL.
2. `connected` emits a trigger when the pool is ready; `createTable` runs immediately.
3. `executeRaw` issues the DDL statement; its `affected: Block<u64>` is converted to `Block<void>` by `check<u64>()`, which then drives `emitDone`.
4. Both `start[server]` and `logReady` receive the `done` signal: the HTTP server starts listening and a log line is emitted.
5. For every `GET /users`: a new track is created; `bodyTrigger` separates the byte stream arrival from the control signals; `queryUsers` streams result rows; `logDataInfos` logs each `Map` row; a fixed `"OK\n"` is streamed back.
6. For every `POST /users`: the JSON body is decoded, parsed, stringified, and echoed verbatim.

### Key Mélodium patterns used

- **`connected` as model source**: fires automatically when the pool connects; no `trigger` input needed.
- **`check<T>()`**: converts `Block<T>` to `Block<void>` when only the event matters, not the value.
- **`stream<T>()`**: converts a `Block<T>` (single value) to `Stream<T>` so it can feed stream-typed inputs like `encode.text`.
- **`bodyTrigger: trigger<byte>()`**: a standard pattern for HTTP handlers that collects the incoming byte stream and emits a `Block<void>` start signal used to gate status and header emission.
