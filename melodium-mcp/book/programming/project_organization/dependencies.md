# Dependencies

Mélodium is not only a language and execution engine, but a dependency manager too.

> Mélodium, as dependency manager, is largely inspired from Rust Cargo.
> If you're already familiar with the way `cargo` work with dependencies,
> you shouldn't discover new things here.

> The dependency and package management in Mélodium is still at its early stage, and things
> may change about it in the future. Information presented here is about actual state and not
> written in stone.

## Dependency list

Every Mélodium project have a dependency list, that it requires to work.
This list is made in the `Compo.toml` file.

A usual dependency list can be:
- `std` `>=0.8.0`
- `http` `>=0.8.0`
- `fs` `>=0.8.0`

Note that the `std` dependency in Mélodium have to be explicitly given.
This is mainly because standard library is in quick evolution and require to rely on precise version.

## Dependency tree

Each project having its own dependency list, using a project means building a dependency tree.
Mélodium is quite cool about dependencies management, but have some rules:
- a project cannot rely directly on other versions of that same project (_but_ multiple versions can appear in the dependency tree);
- circular dependencies are forbidden, meaning a project of a given version cannot appear anywhere in its own dependency tree.
