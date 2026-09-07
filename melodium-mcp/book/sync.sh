#!/usr/bin/env bash
# Refreshes the vendored Mélodium book chapters (this directory) from
# https://gitlab.com/melodium/book. Run manually, from anywhere:
#
#   melodium-mcp/book/sync.sh
#
# This is a deliberate manual step, not a build.rs / CI hook: the book's
# Markdown is embedded into the melodium-mcp binary at compile time via
# `include_dir!`, so what's committed here is exactly what ships. Fetching
# it from git at build time would make builds depend on network access and
# the state of an external repository at build time, which breaks offline
# builds (docs.rs, vendored/air-gapped builds) and reproducibility (the
# same source commit could embed different book content depending on when
# it's built). See README.md in this directory for details.

set -euo pipefail

BOOK_REPO="https://gitlab.com/melodium/book.git"
DEST="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

git clone --quiet --depth 1 "$BOOK_REPO" "$TMP/book"
COMMIT="$(git -C "$TMP/book" rev-parse HEAD)"
DATE="$(date -u +%Y-%m-%d)"

# Drop every previously vendored chapter (but keep this script and README.md)
# so removed/renamed chapters don't linger.
find "$DEST" -name '*.md' ! -name 'README.md' -delete
find "$DEST" -mindepth 1 -type d -empty -delete

(cd "$TMP/book/src" && find . -name '*.md' -print0) | while IFS= read -r -d '' f; do
    mkdir -p "$DEST/$(dirname "$f")"
    cp "$TMP/book/src/$f" "$DEST/$f"
done

sed -i.bak -E \
    "s/^Vendored from commit \`[0-9a-f]+\` of \`melodium\/book\`, refreshed [0-9-]+\.\$/Vendored from commit \`${COMMIT}\` of \`melodium\/book\`, refreshed ${DATE}./" \
    "$DEST/README.md"
rm -f "$DEST/README.md.bak"

echo "Vendored melodium/book@${COMMIT} into $DEST"
echo "Review the diff, then commit melodium-mcp/book/."
