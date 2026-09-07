# Vendored Mélodium book

This directory is a vendored copy of the Markdown chapters (`src/`) from the
[Mélodium book](https://gitlab.com/melodium/book) (the source for
https://doc.melodium.tech/book/en/), excluding images and build
configuration. It is embedded into the `melodium-mcp` binary at compile time
so the `search_book`/`read_book_chapter` tools work offline, without a
runtime dependency on the book's repository or website.

Vendored from commit `51989870feacbea586c016c8ab7ac09bc7eb47ec` of `melodium/book`, refreshed 2026-09-07.

To refresh, run `book/sync.sh` (clones `melodium/book`, replaces this
directory's chapters, and rewrites the line above), then review the diff
and commit. This is a deliberate manual step rather than a build-time
fetch — see `sync.sh` for why.
