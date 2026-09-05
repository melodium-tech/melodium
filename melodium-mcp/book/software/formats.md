# Formats

Mélodium projects can come in three formats:
- a project tree with structure of `*.mel` script files;
- a standalone `*.mel` script file;
- a packaged project as `*.jeu` file.

These three formats have slightly different purposes and are detailed in their own subsection.

## Project Tree

This project format consists of a tree of `*.mel` script files dispatched in directories, where each file and directory are their own _area_.

```
my_project/
├── baz.mel
├── Compo.toml
├── foo
│   └── bar.mel
├── foo.mel
└── main.mel
```

At the root of the project is a `Compo.toml` file, containing the information related to the project itself, such as its name, dependencies, entrypoints, and so on.

This project format is the most useful for development, as it is easily manageable with any code editor and fits with versioning systems like Git.

## Standalone File

This format is, as its name suggest, a single `*.mel` script file.

Everything is contained in a single Mélodium script, including the information about dependencies.

A standalone project file is basically a usual `*.mel` file with special heading inside.
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

The details about standalone files are given in the [Standalone Files](../programming/project_organization/standalone_files.md) chapter.

This project format is useful for quick scripts edition and deployment, prototyping, and to use as system tool.

## Packaged Project

Mélodium have a project package format called Jeu.
It is basically a compressed archive format with some prefixed data for Mélodium and host system handling.

When shipping a project, either for testing, deployment, or any use case better handling one-file program, the Jeu format is indicated.

To package a project as Jeu file, use the `jeu` subcommand:
```sh
melodium jeu build <PROJECT> <OUTPUT_FILE>
```

Jeu files have `.jeu` extension, and are directly usable as programs on systems where Mélodium is installed.

> As Jeu files are already highly compressed files (using the LZMA2 algorithm), it is in general not useful to re-compress it inside other archive formats.
