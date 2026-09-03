# 07: SQL CRUD API

**Concepts introduced:** a `SqlPool` model shared across requests, `fetch` vs. `execute`, combining a database model with an HTTP server model in one program.

A tiny "notes" API backed by PostgreSQL: `POST /notes` stores the request body as plain text, `GET /notes` lists every stored note.

> **Note on verification:** unlike the previous tutorial steps, this example needs a reachable PostgreSQL database. It was type-checked with `melodium check` but **not** run end-to-end against a live database for this tutorial (no Postgres server was available in the environment that wrote it). Point `db_url` at any real Postgres instance to try it for real: the query shapes and wiring follow the same verified patterns as examples 03–06.

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

## Runtime behaviour

1. `connect` is fired once at startup; the `connected` *source* treatment starts a track once the pool is actually ready: `createTable` (and everything downstream, including starting the HTTP server) only runs after that, so no request can race the table's creation.
2. `POST /notes` reduces the body to a single `Block<string>` with `trigger.last` (the same "collapse a one-item stream to a block" idiom as the totals in example 03), wraps it in a `Map` with `std/data/map/block::entry`, and passes it as the single bind parameter to `execute`.
3. `GET /notes` does not build a response and send it once: `fetch`'s `data` output streams each row as soon as it arrives from the database, and each row is turned into one `"id) text\n"` line and written straight into `connection.data`. Nothing is buffered client-side; the HTTP response grows as rows arrive.
4. `SELECT id::text AS id, text FROM notes` casts `id` to text *in SQL*, rather than guessing which native integer type (`i32`? `i64`?) the Postgres driver maps `SERIAL` to: `std/data/map::get<string>` then always matches.

### Key Mélodium patterns used

- **A model shared by every route**: `db` and `server` are instantiated once in `main` and passed down to `createNote`/`listNotes` via model configuration parameters (`[db=db, http_server=server]`), exactly like [06_http_server_api](../06_http_server_api/)'s single `server` model, just with a second one alongside it.
- **`fetch` (Stream<Map> rows) vs. `execute` (single Block<Map> bind, one outcome)**: `fetch` is for reading potentially many rows; `execute` is for one write with one set of parameters.
- **Casting in SQL to dodge a type-mapping guess**: when a value's exact Mélodium type coming back from a driver is uncertain, it is often simpler to coerce it to `string` in the query itself than to guess (and get it wrong silently, since `std/data/map::get<T>` returns `none` on a type mismatch, not an error).
- **`connection.started` for both routes**: following the rule from example 06, since `GET /notes` has no request body at all.

Next: [08_javascript_transform](../08_javascript_transform/) introduces the JavaScript engine, including the field-by-field JSON access that `07_sql_crud_api` and `06_http_server_api` deliberately avoided.
