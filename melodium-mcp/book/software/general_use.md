# General Use

Mélodium, as software, can be essentially resumed as a single executable binary.

The Mélodium binary doesn't require to be at any specific location on the system, and doesn't require more permissions nor privileges than a usual executable program. This said, it have to be in the `PATH` of the user in order to work correctly with standalone Mélodium script and program files.

> If Mélodium has been installed through a system-dedicated installation method these requirements are met, such as with an installer on Windows or package on Linux.

Mélodium software is very similar to what can be found with other programming technologies, such as Python or Java, among others.
The Mélodium executable binary is the _implementation_ of the Mélodium programming language, and receive Mélodium script and program files to _execute_ them.

## Direct Call

Mélodium can be called directly through command line, using the `melodium` command, followed by the name of the script or program file that has to be executed.

By default, the `main` entrypoint will be used, and possible following parameters will be given to the designated treatment.

A Mélodium program can declare any number of entrypoints, and having a `main` one is optionnal. In general, a program with a `main` entrypoint is aimed to be used directly, while projects without `main` entrypoint (either none or multiple others) are libraries.

Calling a Mélodium program is as simple as:
```sh
melodium my_program.mel
```
And it is strictly equivalent to:
```sh
melodium my_program.mel main
```

If arguments needs to be passed to the program, they can be specified as any CLI arguments:
```sh
melodium my_program.mel [ENTRYPOINT] --my_number 42 --my_boolean true
```

To see what entrypoints a Mélodium program proposes, use the `info` command:
```sh
melodium info my_program.mel
```

For more CLI usage details, please refer to the [Command Line Interface](cli.md) section.

## Executable Scripts and Programs

Mélodium scripts files (with `.mel` extension) and packaged files (with `.jeu` extension) can be called directly as-is if they have been designed for this use by the authors.

> On POSIX systems, it require for the file to be marked as executable (`x` permission), and in case of `.mel` file, to start with the `#!/usr/bin/env melodium` shebang line.

This means Mélodium scripts and programs can be called as simply as any shell or general executable file:
```sh
my_program.mel --my_number 42 --my_boolean true
```
