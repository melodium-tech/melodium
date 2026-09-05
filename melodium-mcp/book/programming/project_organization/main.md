# Project Organization

Mélodium projects are organized through a root folder, containing a `Compo.toml` file, and `.mel` files.

```
my_project/
├── Compo.toml
├── baz.mel
├── foo
│   └── bar.mel
├── foo.mel
└── main.mel
```

## Areas

In Mélodium, an _area_ is an element location. It is a concept similar to Rust modules or Java locations.

An area is materialized by a `.mel` file, that contains all its elements.
Subareas are made using a folder having the area name, and creating `.mel` files inside.

```
my_project/
├── Compo.toml
├── baz.mel
├── foo     # folder for subareas
│   └── bar.mel
├── foo.mel # area
└── main.mel
```

In this example, the `foo` area is developed within the `foo.mel` file, and `bar` area is located within the `foo/` folder to be a subarea of `foo`.
They can respectively be called through:
- `root/foo::<element>`
- `root/foo/bar::<element>`

## Current project references

In Mélodium, project code can refer to its own content using the `root` and `local` keywords, and do not use the name of the project itself.

In the example, to refer an element that is present in the `baz` area, the call `root/baz::<element>` must be used.
Similarly, if `foo` area refers to something within `bar`, `local/bar::<element>` could be written.