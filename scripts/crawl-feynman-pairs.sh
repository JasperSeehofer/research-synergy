#!/usr/bin/env bash
# Phase 29: Kuramoto-LBD v03 corpus build
# Crawls all 10 Feynman pair seeds with bidirectional S2 mode at depth 2.
#
# Usage:
#   ./scripts/crawl-feynman-pairs.sh [DB_PATH]
#
# DB_PATH defaults to surrealkv://./data-kuramoto
# Override the binary or forward-citation cap via env vars:
#   BINARY=./target/debug/resyn ./scripts/crawl-feynman-pairs.sh
#   MAX_FWD=200 ./scripts/crawl-feynman-pairs.sh

set -euo pipefail

DB="${1:-surrealkv://./data-kuramoto}"
BINARY="${BINARY:-./target/release/resyn}"
MAX_FWD="${MAX_FWD:-500}"

# 10 Feynman pair seeds (5 evaluable pairs × 2 sides) from
# professional-vault/prototypes/data/feynman_10pair_papers.json (schema v0.2).
seeds=(
  "cond-mat/0203227"   # pair01-A: Dorogovtsev Ising on networks
  "0710.3256"          # pair01-B: Castellano social dynamics
  "cond-mat/0010317"   # pair03-A: Pastor-Satorras SIR
  "cond-mat/0312131"   # pair03-B: Moreno rumour spreading
  "cond-mat/0007235"   # pair04-A: Newman random graphs
  "cond-mat/0205009"   # pair04-B: Newman epidemic disease
  "nlin/0202034"       # pair05-A: Drossel food webs
  "cond-mat/0002374"   # pair05-B: Bouchaud wealth
  "1005.1986"          # pair06-A: Nakao Turing
  "cond-mat/9801289"   # pair06-B: Marsili Zipf
)

echo "DB:      $DB"
echo "Binary:  $BINARY"
echo "max-fwd: $MAX_FWD"
echo "Seeds:   ${#seeds[@]}"
echo ""

for ID in "${seeds[@]}"; do
  echo ""
  echo "=== Seeding $ID ==="
  "$BINARY" crawl \
    --paper-id "$ID" \
    --max-depth 2 \
    --source semantic_scholar \
    --parallel 1 \
    --bidirectional \
    --max-forward-citations "$MAX_FWD" \
    --db "$DB"
done

echo ""
echo "=== All 10 Feynman seeds complete ==="
