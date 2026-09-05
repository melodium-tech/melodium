# Connections

Connections are a specific element, they are the ways by which the events and data are shared among treatments, and ultimately creates the functionnal logic of a program.

Connections are made the most basic element by Mélodium logic, at the core of each treatment nature. Rules are quite soft about them, and the few basic restrictions that applies are:
- a connection cannot link different kinds of data input/output;
- a connection cannot create a cycle (the graph-oriented equivalent of an infinite loop with no termination);
- multiple connections cannot connect to the same treatment input.

These rules exist for determinism and safety: type matching is enforced at build time rather than at runtime; cycles would cause a treatment to feed its own input indefinitely with no way to stop; and requiring a single source per input ensures that the order of data arriving at a treatment is always predictable.

## Inputs

Inputs are the way treatments receive data.
There can be any number of input declared for a given treatment, the only constraint being that when using this treatment, all inputs it have must be connected to some output once.

Most of the treatments wait to receive some data before starting to process anything.

## Outputs

Outputs are the way treatments send data.
As for inputs, there can be any number of output declared for a given treatment, however any output can be connected to as many inputs as needed, or not be used at all.

Most of the treatments process data as long as their main functionnal outputs stay used, and stops when no more following treatments consume it.

## Streaming

Inputs and outputs can be divided in two main categories, the _streaming_ ones and the _blocking_ ones.
The streaming inputs and outputs are basically receiving and sending data as flow.
There can be any amount of data corresponding to the associated type passed through streaming inputs and outputs.
Streaming inputs and outputs are the general case of data transmission.

## Blocking

The blocking inputs and outputs emit and take at most one and only one element of the given data type.
This type of connection is specific for event transmission. It is generally used as trigger to start some processing.

The name "blocking" refers to the idea of a single block of data, not to suspending execution. A `Block<T>` port carries exactly one value, delivered as a whole unit, as opposed to a `Stream<T>` port which carries a continuous sequence of values.
