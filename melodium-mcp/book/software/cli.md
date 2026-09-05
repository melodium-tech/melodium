# Command Line Interface

Mélodium is distributed as a program that have extensive and growing CLI, to see the exhaustive commands and options list:
```shell
melodium help
```

## Run program

Launch a Mélodium program:
```shell
melodium <FILE>
```
or
```shell
melodium run <FILE>
```
or launch a specific command in Mélodium program:
```shell
melodium run <FILE> <CMD> [ARGS…]
```

To see specifics of a program entrypoints:
```shell
melodium run <FILE> <CMD> --help
```

> **Note:** If Mélodium is installed on system, standalone `.mel` files and `.jeu` files are directly callable as-is through command line.

## Program information

See the commands and options of a program:
```shell
melodium info <FILE>
```

## Check code

Check a Mélodium program validity:
```shell
melodium check <FILE>
```

## Generates documentation

Generates documentation, as [mdBook](https://rust-lang.github.io/mdBook/), for a Mélodium package:
```shell
melodium doc --file <FILE> <OUTPUT>
```

## Build `Jeu` files

To build a `Jeu` file from a project:
```shell
melodium jeu build <PROJECT> <OUTPUT_FILE>
```
