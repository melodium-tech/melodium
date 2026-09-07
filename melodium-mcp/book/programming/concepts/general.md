# General Orchestration

Mélodium is a language fully oriented to what happens with data.
As such, in Mélodium, data and signals follows an ensemble of paths that brings it to different kind of treatments and processes.

Unlike many programming languages, Mélodium makes a total abstraction of the instruction order and don't rely on a line-by-line execution scheme.
Every element can run and be shared among different threads depending on the load, data availability, or orchestration optimization decision.

To proceed with those ideas, two major elements exists in Mélodium: _models_ and _treatments_.
Both are essential and can summarize the whole power of Mélodium, and are briefly explained here, more extensive explanations are made in their dedicated [Elements chapters](../elements/main.md).

## Models

Models are long-lived elements that persist for the entire execution of a program. They are the sources from which events occur and data arrives. A filesystem watcher, an HTTP server, or a SQL connection pool are all models. Models can be declared over other models, inheriting their capabilities.

Full syntax and examples are covered in the [Models](../elements/models.md) chapter.

## Treatments

Treatments describe flows of operations applied to data. They can be seen as maps, where paths connect sources to destinations through different processing steps. All treatments within a treatment body can run simultaneously; declaration order has no meaning.

Full syntax and examples are covered in the [Treatments](../elements/treatments.md) chapter.