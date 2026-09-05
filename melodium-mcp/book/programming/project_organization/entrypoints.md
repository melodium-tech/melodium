# Entrypoints

Mélodium projects can have _entrypoints_. As their name suggest, they are the treatments that can be called directly to start a Mélodium program.

A project is not required to have entrypoints, if it is a library as example.
On the opposite, a project can have multiple entrypoints, if that project is a program that can be used in different situations.

## Naming entrypoints

Entrypoints are essentially a name associated with a treatment path, like:
- `server`: `my_project/foo::serve`
- `client`: `my_project/bar::request`
- `main`: `my_project/etc::main`

Entrypoints names have same restriction as treatments names but don't have to be the same as the treatment they designates.

The entrypoint name is expected to be typed in command line, as _commands_ of the program:
```shell
// Starts 'my_program' with 'server' entrypoint
$ my_program.jeu server

// With explicit melodium command
$ melodium my_program.jeu server
```

An exception is made for the `main` entrypoint, that if present, is called directly if no specific entrypoint is given to program.
```shell
// Starts 'my_program' with 'main' entrypoint
$ my_program.jeu

// With explicit melodium command
$ melodium my_program.jeu
```

## Entrypoints parameters

If a treatment used as entrypoint have parameters, they automatically becomes acceptable command arguments.

```mel
// In root/foo
treatment serve(bind: string = "localhost", port: u16) 
{
    /* 
        Implementation…
    */
}
```
With entrypoint `server`: `my_project/foo::serve`.

```shell
$ my_program.jeu server --port 6789 --bind 192.168.55.66
```
