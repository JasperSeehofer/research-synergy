#!/usr/bin/env python3
"""EXP-RS-11 — build TF-IDF cosine semantic-edge graphs from a resyn Louvain export.

Pre-registered protocol (vault: wiki/meta/agentic-experiments-research.md § EXP-RS-11):
load c-TF-IDF vectors from the export's `nodes` field, compute pairwise cosine
similarity, threshold at tau in {0.2, 0.3, 0.4, 0.5}; for each threshold report
(n_nodes, n_edges, n_cc, largest_cc_size, mean_degree). Connectivity precheck:
dynamics may run only where n_cc / N <= 0.05.

Conventions (research-synergy .planning/research/CONVENTIONS.md):
  C-4  node vectors = c-TF-IDF top-50 terms from export-louvain-graph
  C-6  tau sweep {0.2, 0.3, 0.4, 0.5}; precheck n_cc/N <= 0.05
  Edge rule (fixed by this script, appended as convention): undirected edge (i, j),
  i < j, iff cosine(v_i, v_j) >= tau. Isolated nodes are retained and count as
  singleton connected components.

Determinism: no randomness is used anywhere in this script. Connected components
are computed with a hand-rolled union-find (NOT scipy/networkx) so that independent
verification can use a genuinely different code path.

Contamination guard: this script never reads the Feynman benchmark file; the
substrate is built from the export's node vectors alone.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import sys

import numpy as np

DEFAULT_TAUS = [0.2, 0.3, 0.4, 0.5]
PRECHECK_MAX_NCC_FRAC = 0.05  # C-6: proceed to dynamics only where n_cc/N <= 0.05


def load_export(path: str) -> dict:
    with open(path) as f:
        return json.load(f)


def build_tfidf_matrix(nodes: list[dict]) -> tuple[np.ndarray, list[str]]:
    """Dense L2-normalised TF-IDF matrix (N x V) from export nodes.

    Mirrors kuramoto_lbd_v03 cell 13: vocabulary = union of exported terms,
    entries = exported scores, rows L2-normalised.
    """
    vocab = sorted({term for n in nodes for term, _ in n["tfidf_vec"]})
    term_to_idx = {t: i for i, t in enumerate(vocab)}
    mat = np.zeros((len(nodes), len(vocab)), dtype=np.float64)
    for i, n in enumerate(nodes):
        for term, score in n["tfidf_vec"]:
            mat[i, term_to_idx[term]] = score
    norms = np.linalg.norm(mat, axis=1)
    norms = np.maximum(norms, 1e-12)
    return mat / norms[:, None], vocab


class UnionFind:
    def __init__(self, n: int):
        self.parent = list(range(n))
        self.rank = [0] * n

    def find(self, x: int) -> int:
        while self.parent[x] != x:
            self.parent[x] = self.parent[self.parent[x]]
            x = self.parent[x]
        return x

    def union(self, a: int, b: int) -> None:
        ra, rb = self.find(a), self.find(b)
        if ra == rb:
            return
        if self.rank[ra] < self.rank[rb]:
            ra, rb = rb, ra
        self.parent[rb] = ra
        if self.rank[ra] == self.rank[rb]:
            self.rank[ra] += 1


def components(n: int, edges: list[tuple[int, int]]) -> list[int]:
    """Connected-component sizes (descending); isolated nodes are singletons."""
    uf = UnionFind(n)
    for i, j in edges:
        uf.union(i, j)
    sizes: dict[int, int] = {}
    for x in range(n):
        r = uf.find(x)
        sizes[r] = sizes.get(r, 0) + 1
    return sorted(sizes.values(), reverse=True)


def sweep(cos: np.ndarray, taus: list[float]) -> list[dict]:
    n = cos.shape[0]
    iu, ju = np.triu_indices(n, k=1)
    sims = cos[iu, ju]
    results = []
    for tau in taus:
        mask = sims >= tau  # edge rule: cosine >= tau
        edges = list(zip(iu[mask].tolist(), ju[mask].tolist()))
        weights = sims[mask].tolist()
        cc_sizes = components(n, edges)
        n_cc = len(cc_sizes)
        largest = cc_sizes[0] if cc_sizes else 0
        results.append(
            {
                "tau": tau,
                "n_nodes": n,
                "n_edges": len(edges),
                "n_cc": n_cc,
                "largest_cc_size": largest,
                "largest_cc_frac": largest / n if n else 0.0,
                "mean_degree": 2.0 * len(edges) / n if n else 0.0,
                "n_isolated": sum(1 for s in cc_sizes if s == 1),
                "ncc_frac": n_cc / n if n else 1.0,
                "precheck_pass": (n_cc / n) <= PRECHECK_MAX_NCC_FRAC if n else False,
                "_edges": edges,
                "_weights": weights,
            }
        )
    return results


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--input", default="data/research_synergy_pre2015.json")
    ap.add_argument("--output", default=None, help="JSON artifact with stats + edge lists")
    ap.add_argument("--taus", default=",".join(str(t) for t in DEFAULT_TAUS))
    args = ap.parse_args()

    taus = [float(t) for t in args.taus.split(",")]
    with open(args.input, "rb") as f:
        input_sha256 = hashlib.sha256(f.read()).hexdigest()
    export = load_export(args.input)
    nodes = export["nodes"]
    n = len(nodes)

    tfidf_norm, vocab = build_tfidf_matrix(nodes)
    cos = tfidf_norm @ tfidf_norm.T

    print(f"EXP-RS-11 build_tfidf_graph — input: {args.input}")
    print(f"  input sha256:        {input_sha256}")
    print(f"  corpus_fingerprint:  {export.get('corpus_fingerprint', 'N/A')}")
    print(f"  nodes: {n}   vocab: {len(vocab)}")
    print(f"  randomness: none (fully deterministic)")
    print(f"  python {platform.python_version()}  numpy {np.__version__}")
    print()

    results = sweep(cos, taus)

    header = f"{'tau':>5}  {'n_nodes':>7}  {'n_edges':>7}  {'n_cc':>5}  {'largest_cc':>10}  {'cc_frac':>7}  {'mean_deg':>8}  {'isolated':>8}  {'ncc/N':>6}  precheck"
    print(header)
    print("-" * len(header))
    for r in results:
        print(
            f"{r['tau']:>5.2f}  {r['n_nodes']:>7d}  {r['n_edges']:>7d}  {r['n_cc']:>5d}  "
            f"{r['largest_cc_size']:>10d}  {r['largest_cc_frac']:>7.3f}  {r['mean_degree']:>8.2f}  "
            f"{r['n_isolated']:>8d}  {r['ncc_frac']:>6.3f}  "
            f"{'PASS' if r['precheck_pass'] else 'FAIL'}"
        )

    passing = [r["tau"] for r in results if r["precheck_pass"]]
    print()
    print(f"Connectivity precheck (n_cc/N <= {PRECHECK_MAX_NCC_FRAC}): "
          f"{'PASS at tau=' + str(passing) if passing else 'NO tau passes — kill gate fires'}")

    if args.output:
        id_of = [nd["id"] for nd in nodes]
        artifact = {
            "experiment": "EXP-RS-11",
            "input_file": args.input,
            "input_sha256": input_sha256,
            "corpus_fingerprint": export.get("corpus_fingerprint"),
            "edge_rule": "undirected (i<j) iff cosine >= tau; isolated nodes retained",
            "precheck_max_ncc_frac": PRECHECK_MAX_NCC_FRAC,
            "versions": {"python": platform.python_version(), "numpy": np.__version__},
            "sweep": [
                {k: v for k, v in r.items() if not k.startswith("_")} for r in results
            ],
            "graphs": {
                str(r["tau"]): {
                    "edges": [
                        {"src": id_of[i], "dst": id_of[j], "weight": w}
                        for (i, j), w in zip(r["_edges"], r["_weights"])
                    ]
                }
                for r in results
            },
        }
        with open(args.output, "w") as f:
            json.dump(artifact, f, indent=1)
        print(f"Artifact written: {args.output}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
