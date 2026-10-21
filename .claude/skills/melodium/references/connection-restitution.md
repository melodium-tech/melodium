# Connection Restitution Rules

When generating Mélodium source code from a treatment design, connections are not emitted as isolated `A.out -> B.in` edges. They are grouped into **chains** and **fan-out groups** to maximise readability.

---

## Chain syntax recap

The chained form `T.input,output` means: arrive at `T` on port `input`, then depart from `T` on port `output`. A full chain reads left-to-right:

```mel
Self.data -> parse.text,lines -> write.data,done -> Self.result
```

The arrow `->` can use any number of dashes (`->`, `-->`, `--->`…); dash count has no semantic meaning and is used only for visual alignment.

---

## Rule 1 — Inline continuation

A connection is extended inline (on the same line) when all of the following hold:

1. The destination treatment has **exactly one** output port with outgoing connections.
2. That output port connects to **exactly one** successor.
3. Neither the current connection nor the successor has attributes (`#[...]`).

When these conditions are met, append `,out_port -> next.input` to the current line and keep walking. Break the line as soon as any condition fails.

**Result:** linear pipelines collapse to a single line.

```mel
Self.text -> toUtf8.text,encoded -> writeLocal.data,amount -> Self.written_bytes
```

---

## Rule 2 — Fan-out group

When a treatment output connects to **more than one** input (fan-out), or when the single successor has attributes, the chain line is terminated after appending the departing output port, and a **fan-out group** is rendered:

- Each branch of the fan-out becomes its own line.
- Branches are sorted alphabetically by `(receiver_treatment, receiver_input)`.
- Each branch's source name (`T.out`) is **right-padded with spaces** to the same column, so all `->` arrows align vertically within the group.
- Each branch then continues as its own chain (Rule 1 applies recursively).

```mel
connection.data -> trigger.stream,start --> status.trigger,emit -> connection.status
                       trigger.start --------> headers.trigger,emit -> connection.headers
                       trigger.start ---------> logConn.trigger
```

Here `trigger.start` fans out to three receivers. The first branch (`status`) is sorted first, the others follow. All three source strings are padded so their `->` arrows land at the same column.

---

## Rule 3 — Attribute break

A connection that carries `#[...]` attributes **always** starts its own line. It cannot be an inline continuation of a predecessor, and its attributes are emitted on the line(s) immediately before it, at the same indentation. Inside a fan-out group, attribute lines are indented to align under the group's source name.

---

## Rule 4 — Root ordering

A **root** is any connection that must start a new chain line (i.e. it is not an inline continuation). Roots are emitted in this order:

1. `Self`-sourced connections first (connections whose source is the hosting treatment's own input ports).
2. All others, sorted alphabetically by `(src_treatment, src_output)`, then by `(dst_treatment, dst_input)`.

---

## Rule 5 — Multiple output ports

When a destination treatment has **multiple distinct output ports** each with successors, the chain ends at that treatment (no single port can be selected for inlining), and each output port is rendered as a separate fan-out group in alphabetical port order.

```mel
A.out -> B.in
B.first -> C.in
B.second -> D.in
```

becomes:

```mel
A.out -> B.in,first -> C.in
         B.second   -> D.in
```

---

## Rule 6 — Shared fan-in (two outputs → same treatment, different inputs)

Because each input has exactly one incoming connection, two outputs can never share the same input. They can arrive at the same *treatment* on different inputs. In this case each arriving connection is its own chain root:

```mel
A.out -> B.in,out -> D.in_1
A.out -> C.in,out -> D.in_2
```

Both `A.out -> B.in` and `A.out -> C.in` are roots (A fans out). Each then continues as its own chain. `D` appears twice as a destination — once per input — which is correct and unambiguous.

---

## Summary table

| Situation | Behaviour |
|---|---|
| Single successor, no attributes | Extend inline: `T.input,output -> next.input` |
| Fan-out (N > 1 receivers on same port) | Emit group; pad source names; sort branches A–Z |
| Single successor has attributes | Break line; emit group (single-branch) |
| Connection has attributes | Always new line; attributes printed above it |
| Multiple output ports on destination | End chain; render each port as separate group |
| Two chains arrive at same treatment on different inputs | Each is an independent root |
| `Self`-sourced connections | Always rendered before internal-treatment roots |
