#!/usr/bin/env bash
# crawl-feynman-seeds.sh
# Crawl a curated set of Feynman-diagram / cond-mat seeds using SemanticScholar
# in bidirectional mode (forward + backward citations).
#
# Usage:
#   ./scripts/crawl-feynman-seeds.sh [DB_PATH]
#
# DB_PATH defaults to surrealkv://./data-feynman

set -euo pipefail

DB="${1:-surrealkv://./data-feynman}"
BINARY="${BINARY:-./target/release/resyn}"

# Feynman / condensed-matter seeds (pre-2015 cond-mat papers with rich forward-citation graphs).
seeds=(
  "1411.4903"   # Kitaev honeycomb model paper
  "cond-mat/0010317"  # Epidemic spreading in scale-free networks (Pastor-Satorras & Vespignani)
)

echo "Using DB: $DB"
echo "Using binary: $BINARY"
echo ""

for ID in "${seeds[@]}"; do
  echo ""
  echo "=== Seeding $ID ==="
  "$BINARY" crawl --paper-id "$ID" --max-depth 2 --source semantic_scholar --parallel 1 --bidirectional --db "$DB"
done

echo ""
echo "=== All seeds complete ==="
