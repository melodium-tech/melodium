# Treatments


Treatments are the main element of the language, they can take parameters. Unlike functions, which list instructions to execute and apply changes on variables, treatments describes flows of operations that applies on data. It can be seen as a map, on which paths connects from sources to destinations, browsing through different locations with different purposes. Order of declaration has no importance, treatments will run when there are data ready to be processed, and all treatments can be considered as running simultaneously.

Within treatments are other treatments, that also take parameters, inputs, and provide outputs to the hosting treatment. Treatments are declared once, and then can be connected as many times as needed.
```mel
treatment myTreatment(var foo: u64, var bar: f64)
{
    treatmentA(alpha=foo)
    treatmentB(beta=bar, gamma=0.01)

    treatmentA.output --> treatmentB.input
}
```
![myTreatment graph](images/myTreatment.png)

## Connections

Connections are basically paths data will follow. 
Connection can connect treatments outputs to inputs, but also refers to the inputs and outputs of the hosting treatment itself.
A connection always links an output and an input of the same type.

`Self` refers to the hosting treatment's own ports. It is used in connections to wire the treatment's declared inputs and outputs into its internal graph.

```mel
use fs/local::writeLocal
use std/text/convert/string::toUtf8

treatment writeText(filename: string)
  input  text:          Stream<string>
  output written_bytes: Stream<u128>
{
    writeLocal(path=filename)
    toUtf8()

    Self.text -> toUtf8.text,encoded -> writeLocal.data,amount -> Self.written_bytes
}
```

_Reference for [writeLocal](https://doc.melodium.tech/latest/en/fs/local/writeLocal.html), [toUtf8](https://doc.melodium.tech/latest/en/std/text/convert/string/toUtf8.html)_

![writeText graph](images/writeText.png)


Multiple connections from the same element are totally legal, however overloading a treatment input or a host treatment output (`Self`) is forbidden.
Also, while omitting usage of a treatment output is legal, every input must be satisfied.
Finally, all host treatment outputs must be satisfied.

Inputs and outputs (and so connections) are either streaming or blocking. A _streaming_ connection `Stream<…>` is expected to send continuously values from the specified type.
A _blocking_ connection `Block<…>` is expected to send all-at-once.
This distinction mainly rely on the core treatments that are used and the intrinsic logic applied on data.
What developer should keep in mind is that _streaming_ is the default unless _blocking_ is required.

A specific kind of connection using the data type `void` exists. It is useful for transmitting information that something happens or should be triggered, schedule events, `Block<void>`; or to indicate continuation of something that doesn't convey data by itself, `Stream<void>`. 