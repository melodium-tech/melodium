# Compo.toml

`Compo.toml` file is the very file making a folder being a Mélodium project.

Its structure is really basic, containing few informations about the project:
```toml
name = "my_project"
version = "0.0.1"

[dependencies]
encoding = "0.8.0"
fs = "0.8.0"
http = "0.8.0"
javascript = "0.8.0"

[entrypoints]
main = "my_project/main::main"
```

## Name and version

The `name` field contains the very name of the project, that will be exposed in repository and used by dependents projects to call it.

## Dependencies

The `dependencies` list contains the names of dependencies and version requirements, as explained in the dedicated chapter.

## Entrypoints

The `entrypoints` list contains the named entrypoints and targeted treatments.
