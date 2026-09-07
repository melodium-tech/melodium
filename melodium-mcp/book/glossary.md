# Glossary

**Area**
A namespace unit in Mélodium, materialized by a `.mel` file. Subareas are created by placing `.mel` files inside a folder with the parent area's name. Elements within the same project are referenced using `root/area::element` or `local/subarea::element`.
See: [Project Organization](./programming/project_organization/main.md)

**Block**
A port kind written `Block<T>` that carries exactly one value of type `T`. Used for single events, triggers, and configuration values. The name refers to a single block of data, not to suspending execution.
See: [Connections](./programming/concepts/connections.md)

**Configuration parameter**
A parameter written in square brackets (`[param: Type]`) on a treatment declaration, used to pass a model instance or fixed resource into a treatment. Configuration parameters are always constant.
See: [Parameters](./programming/parameters.md)

**Connection**
A directed link from an output port to an input port, written with `->`. Connections define the data flow between treatment instances inside a treatment body. A connection must link ports of the same type and kind, cannot create cycles, and a given input may only have one connection.
See: [Connections](./programming/concepts/connections.md)

**Context**
Track-wide data prefixed with `@`, provided by a source treatment and implicitly available to any treatment in the same track that declares `require @ContextName`. Unlike parameters, contexts do not need to be explicitly passed down the call chain.
See: [Contexts](./programming/elements/contexts.md)

**Entrypoint**
A named treatment that can be invoked directly from the command line to start a program. Declared in `Compo.toml`. The `main` entrypoint is called when no name is given. Treatment parameters become CLI arguments automatically.
See: [Entrypoints](./programming/project_organization/entrypoints.md)

**Function**
A pure, side-effect-free callable prefixed with `|`, used to compute parameter values. Functions are executed once at program initialization when used as `const` parameters, or once per track creation when used as `var` parameters.
See: [Functions](./programming/elements/functions.md)

**Model**
A stateful element that lives for the entire program execution. Models are the sources of events and tracks: they are the starting points from which data enters the system. A model is declared by extending a library model and configuring its parameters.
See: [Models](./programming/elements/models.md)

**Self**
A keyword used inside a treatment body to refer to the hosting treatment's own input and output ports. `Self.input_name` and `Self.output_name` wire the treatment's declared ports into its internal connection graph.
See: [Treatments](./programming/elements/treatments.md)

**Stream**
A port kind written `Stream<T>` that carries a continuous, unbounded sequence of values of type `T`. The general case for data transmission in Mélodium.
See: [Connections](./programming/concepts/connections.md)

**Track**
The complete set of treatment instances and connections created together from a single model event. Each event from a model spawns a new independent track. All tracks run concurrently, each with its own data and context.
See: [Tracks](./programming/concepts/tracks.md)

**Trait**
An intrinsic capability of a type, such as the ability to be added (`Add`), compared (`PartialEquality`), or converted to a string (`ToString`). Traits are used to constrain generic type parameters.
See: [Traits](./programming/traits/main.md)

**Treatment**
The primary computation unit in Mélodium. A treatment declares inputs, outputs, and a body of other treatment instances connected together. Treatments do not execute line by line; all inner treatments run concurrently when data is available.
See: [Treatments](./programming/elements/treatments.md)

**void**
A core type that carries no value. `Block<void>` is used as a trigger (signals that something happened), and `Stream<void>` is used as a continuation indicator (signals ongoing activity without conveying data).
See: [Core types](./programming/core_types.md)
