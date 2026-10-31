# Connection Layout Rules

Restitution (`melodium_lang::restitution`, used by Compositeur saves and the migrators) writes the connections of a treatment body as a drawing of the graph they form. Code written by hand or generated should follow the same layout: it reads the same way, and a later restitution keeps it unchanged.

---

## Chain syntax recap

`T.input,output` arrives at `T` on `input` and leaves it from `output`:

```mel
Self.data -> parse.text,lines -> write.data,done -> Self.result
```

An arrow can have any number of dashes (`->`, `-->`, `--->`). Dashes carry no meaning and are only used to align destinations.

A chain never continues through `Self`: `Self` as destination receives the treatment outputs, `Self` as source gives its inputs.

---

## Graph terms

- **Nodes**: the instanciated treatments, `Self` as source and `Self` as destination.
- **Rank** of a node: length of the longest path reaching it. `Self` inputs and treatments receiving nothing (`startup()`) have rank 0.
- **Anchor** of a node: the connection entering it from its predecessor of highest rank (the first in order when several have that rank). A node is crossed by one line only, the line of its anchor, so it appears after everything it receives from.

When connections form a cycle (a server connection receiving the response computed from its own request), the cycle is broken at the node where it closes: the node receiving the most connections from within the cycle, unless a node of the cycle is already reached from outside it, which then comes first. The connections closing the cycle are written as plain destinations.

---

## Rule 1: every connection is written under its source

All connections leaving a node form its group, ordered by output port, then destination name (`Self` last), then input port.

- When the node is reached by its anchor, the first connection of the group without attributes continues the line: `-> node.input,output -> next.input`.
- Every other connection of the group starts a new line at the column of the node's name, repeating `node.output`.
- A connection that is not the anchor of its destination ends the line at that destination (`-> other.input`), even if `other` gives connections itself.

```mel
connection.data -> trigger.stream,start -> headers.trigger,emit -> connection.headers
                   trigger.start --------> logConn.trigger
                   trigger.start --------> status.trigger,emit -> connection.status
```

## Rule 2: destinations of a group are aligned

Within a group, arrows are lengthened so every destination starts on the same column. The shortest arrow of the group is `->`.

```mel
Self.label ---> log.label
Self.message -> log.message
```

## Rule 3: attributes start their own lines

A connection with attributes never continues a line. Its attributes are written on the lines right above it, at the column of its source.

```mel
Self.data --> a.in,out -> c.in
              #[note("x")]
              a.out ----> b.in
```

## Rule 4: lines stay within the width

When the group of a node does not fit within the width (100 columns by default), the line ends at the node (`-> node.input`) and the group is written later at the body indentation, starting with `node.output`, once the lines of the current root are written.

```mel
Self.data -> first.input,output -> second.input
second.other --> fourth.input
second.output -> third.input
```

## Rule 5: order of lines

1. The group of `Self` inputs.
2. Groups of nodes receiving nothing, by name.
3. Groups of nodes receiving only connections closing cycles, by name.

Each group is followed by everything drawn from it, depth first, and then by the groups moved by rule 4.

---

## Declarations

Instanciations are declared in the order their names first appear in the connections, then the unconnected ones by name. A declaration fits on one line when it can, parameters included:

```mel
status: emit<HttpStatus>(value = |ok())
```

Otherwise each parameter is on its own line.
