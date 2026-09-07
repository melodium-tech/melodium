# 03: Text & Files

**Concepts introduced:** file I/O (`fs`), regular expressions (`regex`), text composition (`std/text/compose`), building and running local `melodium run` end to end.

**Book:** [Error Handling](https://doc.melodium.tech/book/en/programming/error_handling.html) (the `readTextLocal.failed` pattern below is exactly the pattern this chapter documents).

Reads a text file line by line, keeps the lines that match a regex pattern, and writes a small report summarising how many (non-blank) lines were read and how many matched.

## What it does

```
melodium run Compo.toml --input_file sample.txt --pattern "Mélodium"
```

- Reads `sample.txt` (shipped alongside this example) and splits it into lines.
- Logs every line matching the `pattern` regex.
- Writes `report.txt` with the total line and match counts.

## How it is built

No models here: file reading, regex matching, and text composition are all stateless treatments.

### Data flow

```
readTextLocal ──▶ split + flatten ──▶ trim ──▶ drop blank lines ──┬──▶ regex match ──▶ filter ──▶ log + count
                                                                    └──▶ count (total lines)
                                                                                          ↓
                                                              combine totals + pattern ──▶ format ──▶ writeTextLocal
```

### Reference

- **`fs`**: [readTextLocal](https://doc.melodium.tech/latest/en/fs/local/readTextLocal.html), [writeTextLocal](https://doc.melodium.tech/latest/en/fs/local/writeTextLocal.html)
- **`regex`**: [matches](https://doc.melodium.tech/latest/en/regex/matches.html)
- **`std`**: [startup](https://doc.melodium.tech/latest/en/std/engine/util/startup.html), [logInfoMessage](https://doc.melodium.tech/latest/en/std/engine/log/logInfoMessage.html), [logInfos](https://doc.melodium.tech/latest/en/std/engine/log/logInfos.html), [emit](https://doc.melodium.tech/latest/en/std/flow/emit.html), [stream](https://doc.melodium.tech/latest/en/std/flow/stream.html), [filter](https://doc.melodium.tech/latest/en/std/flow/filter.html), [count](https://doc.melodium.tech/latest/en/std/flow/count.html), [trigger](https://doc.melodium.tech/latest/en/std/flow/trigger.html), [flatten](https://doc.melodium.tech/latest/en/std/flow/vec/flatten.html), [split](https://doc.melodium.tech/latest/en/std/text/compose/split.html), [trim](https://doc.melodium.tech/latest/en/std/text/compose/trim.html), [format](https://doc.melodium.tech/latest/en/std/text/compose/format.html), [StringMap](https://doc.melodium.tech/latest/en/std/data/string_map/StringMap.html), [blockEntry](https://doc.melodium.tech/latest/en/std/data/string_map/block/entry.html), [blockInsert](https://doc.melodium.tech/latest/en/std/data/string_map/block/insert.html), [toString](https://doc.melodium.tech/latest/en/std/conv/toString.html), [not](https://doc.melodium.tech/latest/en/std/ops/bin/not.html), [exact](https://doc.melodium.tech/latest/en/std/text/compare/exact.html)

## Runtime behaviour

1. `readTextLocal` streams the file's raw content in chunks; `split(delimiter="\n")` + `flatten` turns that into a stream of lines: the same "split then flatten" idiom used to decode any delimited stream, not just files.
2. Splitting on `"\n"` leaves one trailing empty piece after the file's last newline, so a small `exact(pattern="") + not + filter` chain drops blank lines before anything else runs.
3. Each line is tested against `pattern` with `regex::matches`; the resulting boolean stream drives `filter`, whose `accepted` branch is both logged and counted.
4. Totals are computed by a small local treatment, `finalCount<T>`, reused for both the line count and the match count: `count` numbers every element as it streams by, and `trigger.last` collapses that running count to its final value once the stream ends.

### Key Mélodium patterns used

- **`split` + `flatten`**: turn any delimited stream (file content, HTTP body, …) into a stream of individual pieces.
- **Filtering out unwanted elements**: `compare/exact` + `std/ops/bin::not` + `filter` is the general pattern for "keep everything except X"; the same shape as "keep only lines matching a pattern" a few lines below, just inverted.
- **Aggregating a stream to one value**: `count` + `trigger.last` is the standard way to turn "how many things flowed through" into a single number available once the stream is finished.

Next: [04_json_toolkit](../04_json_toolkit/) introduces structured JSON data and maps.
