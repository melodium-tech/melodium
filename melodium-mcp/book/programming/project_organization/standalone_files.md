# Standalone Files

Mélodium is able to handle standalone `.mel` files. Opposite to `.jeu` files that are packaged Mélodium applications, `.mel` standalone files aims to stay as simple scripts, for use cases as quick deployment, small tooling, or administration helpers.

Standalone Mélodium files are really usual `.mel` files with a special heading.

```mel
#!/usr/bin/env melodium
#! name = my_project_name
#! version = 0.1.0
#! require = std:0.8.*

/*

    Just a usual Mélodium script afterwards.

    …
*/

```

They always start with the `#!/usr/bin/env melodium` shebang, allowing them to be used as system script.
This shebang is mandatory as it is also used by Mélodium engine to ensure script was designed to be called as-is.

`name` and `version` fields works the same as in `Compo.toml`. `require` is expected to be the list of dependencies, separated by spaces, in with the `<name>:<version_requirement>` format.
`require` can also be repeated multiple times if needed.

Unlike other Mélodium projects, standalone Mélodium files are _required_ to have one and only one entrypoint, that is always set up to the `main` treatment. All the characteristics relevant to treatement parameters and CLI arguments are applicable as for any entrypoint.