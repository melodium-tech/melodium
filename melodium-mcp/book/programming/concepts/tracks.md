# Tracks

Tracks are the most implicit thing in Mélodium. When models are instantiated and treatments connected together, it creates a potential _track_. The track is the whole ensemble of treatments and flows between them, that are created together, live together, and disappear together.

A track always takes its origin from a model, which requests its initialization when needed and as many times as needed: for each file found, each incoming connection, or whatever the model's purpose proposes. Each track follows the same defined chain of treatments, but runs independently. This is one of the core elements providing Mélodium its strong scalability.

For example, an HTTP server model creates one new track for every incoming request. Each track processes its request independently and concurrently, without any shared state with the others:

```mel
use http/server::HttpServer
use http/server::connection
use http/method::|get as |methodGet

// Each incoming GET /ping request spawns its own independent track.
treatment myApp[http_server: HttpServer]() {
    connection[http_server=http_server](method=|methodGet(), route="/ping")
    respond()

    connection.data -> respond.data,result -> connection.data
}
```

All tracks share the same model (the `HttpServer` instance) as their source, but their data flows are entirely separate.
