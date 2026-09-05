# Parameters

Treatments and models declares parameters. Parameters are like in any language: elements given by the caller to set up behavior of the model or treatment.

# Const and var

In Mélodium, parameters can be either constant or variable, respectively declared with keywords `const` and `var`.
A _constant_ parameter designates something that will keep the same value during all the execution, on all tracks generated through the given call. They are used mostly to configure models, that have all parameters required to be constant.
A _variable_ parameter designates something that may have different values on each track generated.

While a constant can be used to set up constant and variable parameters, variable elements (parameters but also contexts) can only be used to set up other variables.

# Configuration parameters

Treatments and models can also declare _configuration parameters_, written in square brackets (`[param: Type]`). These are used to pass models (or other fixed resources) into a treatment, and are always constant. They are set once when the treatment is wired up and do not change at runtime.

```mel
// database is a configuration parameter: it receives a model instance.
treatment userData[database: SqlPool]()
  input  user_id: Block<string>
  output result:  Block<Option<string>>
{
    // database is available here as a fixed resource.
}
```

Model parameters are by definition always `const`, since a model lives for the full duration of the program and its configuration cannot change between tracks.
