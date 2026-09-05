# Models

Models are elements that live through the whole execution of a program. They are declared by extending an existing library model and configuring its parameters.

A simple example using an HTTP server model:

```mel
use http/server::HttpServer

model MyServer() : HttpServer {
    bind = "0.0.0.0"
    port = 8080
}
```
_Reference for [HttpServer](https://doc.melodium.tech/latest/en/http/server/HttpServer.html)_

The `MyServer` model extends `HttpServer` from the `http` package, fixing the bind address and port. It will live for the entire program execution, accepting connections as long as the program runs.

Models can also expose parameters that callers can override. Here is a database connection pool with a configurable maximum:

```mel
use sql::SqlPool

model MyDatabase(const max: u32 = 5) : SqlPool
{
    min_connections = 1
    max_connections = max
    url = "postgresql://my-user@my-server:4321/my_database"
}
```
_Reference for [SqlPool](https://doc.melodium.tech/latest/en/sql/SqlPool.html)_

Models are instantiated by treatments in their prelude.

```mel
use sql::fetch
use sql::SqlPool
use std/data/map/block::entry
use std/data/map/block::get
use std/data/map::Map
use std/flow::trigger


treatment myApp()
  /*
    When model is instantiated, it is made available within the
    treatment as if it where given as configuration parameter.
  */
  model database: MyDatabase(max=3)
  input user_id:       Block<string>
  output user_name:    Block<Option<string>>
  output user_level:   Block<Option<u32>>
  output user_failure: Stream<string>
{
    userData[database=database]()

    Self.user_id -> userData.user_id,user_name -> Self.user_name
                    userData.user_level --------> Self.user_level
                    userData.errors ------------> Self.user_failure
}

/*
  Model can be given as configuration parameter.
*/
treatment userData[database: SqlPool]()
  input user_id:     Block<string>
  output user_name:  Block<Option<string>>
  output user_level: Block<Option<u32>>
  output errors:     Stream<string>
{

    /*
      And then passed again as configuration parameter.
    */
    fetchUser: fetch[sql_pool=database](
      sql = "SELECT name, level IN users WHERE id = ? LIMIT 1",
      bindings = ["user_id"]
    )
    entry<string>(key="user_id")
    trigger<Map>()

    Self.user_id -> entry.value,map -> fetchUser.bind,data -> trigger.stream

    getUserName: get<string>(key="name")
    getUserLevel: get<u32>(key="level")

    trigger.first --> getUserName.map,value -> Self.user_name
    trigger.first -> getUserLevel.map,value -> Self.user_level

    fetchUser.errors -> Self.errors
}
```
_Reference for [fetch](https://doc.melodium.tech/latest/en/sql/fetch.html), [entry](https://doc.melodium.tech/latest/en/std/data/map/block/entry.html), [get](https://doc.melodium.tech/latest/en/std/data/map/block/get.html), [trigger](https://doc.melodium.tech/latest/en/std/flow/trigger.html)_

![myApp graph](images/myApp.png)  
_myApp_

![userData graph](images/userData.png)  
_userData_

In most cases, models are instantiated internally by treatments and not exposed, user developer can make direct call on model-dependent treatments without instantiating its own, just giving required parameters to the sequence. The cases where user may give its own defined model is to configure elements such as external software connections or interfaces.