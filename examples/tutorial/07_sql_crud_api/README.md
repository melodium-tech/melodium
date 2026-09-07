# 07: SQL CRUD API

**Concepts introduced:** a `SqlPool` model shared across requests, `fetch` vs. `execute`, combining a database model with an HTTP server model in one program.

**Book:** [Models](https://doc.melodium.tech/book/en/programming/elements/models.html) (its own `SqlPool` example is essentially `AppDb` below), [Parameters](https://doc.melodium.tech/book/en/programming/parameters.html) (configuration parameters, e.g. `[db: SqlPool]`).

A tiny "notes" API backed by PostgreSQL: `POST /notes` stores the request body as plain text, `GET /notes` lists every stored note.

> **Requirement:** a reachable PostgreSQL database.

## What it does

```
melodium run Compo.toml --db_url postgresql://user@localhost/notes_db

curl -X POST http://127.0.0.1:8080/notes -d "buy milk"
curl http://127.0.0.1:8080/notes
# 1) buy milk
```

## How it is built

| Model | Type | Purpose |
|---|---|---|
| `db` | `SqlPool` | Connection pool to PostgreSQL, shared across every request |
| `server` | `HttpServer` | HTTP listener bound to localhost |

### Data flow

```
startup ─▶ connect ─▶ connected ─▶ createTable ─▶ start (HTTP)
                                                       │
                            POST /notes ──▶ insertNote ──▶ execute (INSERT)
                            GET  /notes ──▶ listRows    ──▶ fetch (SELECT), streamed row by row
```

### Reference

- **`encoding`**: [decode](https://doc.melodium.tech/latest/en/encoding/decode.html), [encode](https://doc.melodium.tech/latest/en/encoding/encode.html)
- **`http`**: [HttpServer](https://doc.melodium.tech/latest/en/http/server/HttpServer.html), [start](https://doc.melodium.tech/latest/en/http/server/start.html), [connection](https://doc.melodium.tech/latest/en/http/server/connection.html), [|get](https://doc.melodium.tech/latest/en/http/method/|get.html), [|post](https://doc.melodium.tech/latest/en/http/method/|post.html), [|ok](https://doc.melodium.tech/latest/en/http/status/|ok.html), [HttpStatus](https://doc.melodium.tech/latest/en/http/status/HttpStatus.html)
- **`net`**: [|localhost_ipv4](https://doc.melodium.tech/latest/en/net/ip/|localhost_ipv4.html), [|from_ipv4](https://doc.melodium.tech/latest/en/net/ip/|from_ipv4.html)
- **`sql`**: [SqlPool](https://doc.melodium.tech/latest/en/sql/SqlPool.html), [connect](https://doc.melodium.tech/latest/en/sql/connect.html), [connected](https://doc.melodium.tech/latest/en/sql/connected.html), [executeRaw](https://doc.melodium.tech/latest/en/sql/executeRaw.html), [execute](https://doc.melodium.tech/latest/en/sql/execute.html), [fetch](https://doc.melodium.tech/latest/en/sql/fetch.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html), [logErrorMessage](https://doc.melodium.tech/latest/en/std/engine/log/logErrorMessage.html), [logError](https://doc.melodium.tech/latest/en/std/engine/log/logError.html), [logErrors](https://doc.melodium.tech/latest/en/std/engine/log/logErrors.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html), [stream](https://doc.melodium.tech/latest/en/std/flow/stream.html), [check](https://doc.melodium.tech/latest/en/std/flow/check.html), [trigger](https://doc.melodium.tech/latest/en/std/flow/trigger.html), [format](https://doc.melodium.tech/latest/en/std/text/compose/format.html), [StringMap](https://doc.melodium.tech/latest/en/std/data/string_map/StringMap.html), [|map](https://doc.melodium.tech/latest/en/std/data/string_map/|map.html), [Map](https://doc.melodium.tech/latest/en/std/data/map/Map.html), [|mmap](https://doc.melodium.tech/latest/en/std/data/map/|map.html), [mapGet](https://doc.melodium.tech/latest/en/std/data/map/get.html), [blockMapEntry](https://doc.melodium.tech/latest/en/std/data/map/block/entry.html), [entry](https://doc.melodium.tech/latest/en/std/data/string_map/entry.html), [insert](https://doc.melodium.tech/latest/en/std/data/string_map/insert.html), [unwrapOr](https://doc.melodium.tech/latest/en/std/ops/option/unwrapOr.html)

## Runtime behaviour

1. `connect` is fired once at startup; the `connected` *source* treatment starts a track once the pool is actually ready: `createTable` (and everything downstream, including starting the HTTP server) only runs after that, so no request can race the table's creation.
2. `POST /notes` reduces the body to a single `Block<string>` with `trigger.last` (the same "collapse a one-item stream to a block" idiom as the totals in [03_text_and_files](../03_text_and_files/)), wraps it in a `Map` with `std/data/map/block::entry`, and passes it as the single bind parameter to `execute`. `execute`'s SQL uses the default `?` placeholder (`bind_symbol`); for a PostgreSQL connection (`postgres://` or `postgresql://`) it is automatically rewritten to `$1`, `$2`, … before reaching the driver, so the `?` in `INSERT INTO notes (text) VALUES (?)` never has to be written as `$1` by hand.
3. `GET /notes` does not build a response and send it once: `fetch`'s `data` output streams each row as soon as it arrives from the database, and each row is turned into one `"id) text\n"` line and written straight into `connection.data`. Nothing is buffered client-side; the HTTP response grows as rows arrive.
4. `SELECT id::text AS id, text FROM notes` casts `id` to text *in SQL*, rather than guessing which native integer type (`i32`? `i64`?) the Postgres driver maps `SERIAL` to: `std/data/map::get<string>` then always matches.

### Key Mélodium patterns used

- **A model shared by every route**: `db` and `server` are instantiated once in `main` and passed down to `createNote`/`listNotes` via model configuration parameters (`[db=db, http_server=server]`), exactly like [06_http_server_api](../06_http_server_api/)'s single `server` model, just with a second one alongside it.
- **`fetch` (Stream<Map> rows) vs. `execute` (single Block<Map> bind, one outcome)**: `fetch` is for reading potentially many rows; `execute` is for one write with one set of parameters.
- **Casting in SQL to dodge a type-mapping guess**: when a value's exact Mélodium type coming back from a driver is uncertain, it is often simpler to coerce it to `string` in the query itself than to guess (and get it wrong silently, since `std/data/map::get<T>` returns `none` on a type mismatch, not an error).
- **`connection.started` for both routes**: following the rule from [06_http_server_api](../06_http_server_api/), since `GET /notes` has no request body at all.

Next: [08_javascript_transform](../08_javascript_transform/) introduces the JavaScript engine, including the field-by-field JSON access that `07_sql_crud_api` and `06_http_server_api` deliberately avoided.
