# Contexts

Contexts are data available through a whole track.
Unlike parameters, they inherently exists and are accessible by all treatments requiring them, as long as the source of the track they are in provide it.

The calling treatments don't have to explicitly pass any context to their inner called treatments.
Contexts have special naming convention, starting with `@`.

## Context providing and requirement

Sources provide contexts, and treatments connected after that source can require the context to access it.

> Requiring a context means the treatment can _only_ be used in a track coming from a source providing such context.

The following minimal example shows the pattern: a source treatment provides `@ConnectionId`, and a downstream treatment requires it without being explicitly passed anything:

```mel
// A context carrying a connection identifier for the current track.
// In practice, contexts like this are provided by library models (HTTP, files, etc.).

treatment logRequest()
  require @ConnectionId
  input  data:   Stream<byte>
  output result: Stream<byte>
{
    // @ConnectionId[id] accesses the "id" field of the context.
    // Its value comes from the track source, not from a parameter.
    process(connection_id = @ConnectionId[id])

    Self.data -> process.data,result -> Self.result
}
```

`logRequest` can only be used inside a track whose source provides `@ConnectionId`. Treatments further down the chain do not need to pass it explicitly: it is available to any treatment in the track that declares `require @ConnectionId`.

Below is a real-world example using the HTTP server, where `@HttpRequest` is provided by the `connection` source treatment:

```mel
use http/server::HttpServer
use http/server::connection
use http/method::|get as |methodGet
use http/server::@HttpRequest
use http/status::|ok
use http/status::HttpStatus
use std/data/string_map::|get
use std/data/string_map::|map
use std/data/string_map::StringMap
use std/flow::trigger
use std/flow::emit

treatment myApp[http_server: HttpServer]() {
    connection[http_server=http_server](method=|methodGet(), route="/user/:user")
    actionGetUser()

    connection.data -> actionGetUser.data,result -> connection.data
    
    trigger<byte>()
    emitHeaders: emit<StringMap>(value=|map([]))
    emitStatus: emit<HttpStatus>(value=|ok())
    
    actionGetUser.result -> trigger.stream,start -> emitHeaders.trigger,emit -> connection.headers
                            trigger.start --------> emitStatus.trigger,emit --> connection.status
}

treatment actionGetUser()
  require @HttpRequest
  input  data:   Stream<byte>
  output result: Stream<byte>
{
    getUser(user_id = |get(@HttpRequest[parameters], "user"))

    Self.data -> getUser.data,result -> Self.result
}

treatment getUser(user_id: Option<string>)
  input  data  : Stream<byte>
  output result: Stream<byte>
{
    // Do some stuff…
}
```
_Reference for [HttpServer](https://doc.melodium.tech/latest/en/http/server/HttpServer.html), [connection](https://doc.melodium.tech/latest/en/http/server/connection.html), [|get (http method)](https://doc.melodium.tech/latest/en/http/method/|get.html), [@HttpRequest](https://doc.melodium.tech/latest/en/http/server/@HttpRequest.html),[|get (map access)](https://doc.melodium.tech/latest/en/std/data/string_map/|get.html)_

![myApp graph](images/myApp_context.png)  
_myApp_

![actionGetUser graph](images/actionGetUser_context.png)  
_actionGetUser_

As contexts comes at track creation, they are inherently _variable_ data.
