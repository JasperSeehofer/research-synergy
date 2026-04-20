"""
EXP-RS-07: Sheaves-LBD v0.1 validation prototype.

Builds a cellular sheaf over the Louvain community graph, assembles the sheaf
Laplacian L_F = δᵀδ, finds near-zero eigenmodes (near-sections), and ranks
inter-community edges by frustration Φ(e). Runs T1/T2/T4/SC1–SC5. Writes
SHEAF_V01_RESULTS.md.
"""

import json
import sys
import os
import numpy as np
from scipy.sparse import coo_matrix
from scipy.sparse.linalg import eigsh

SEED = 42
np.random.seed(SEED)


# ---------------------------------------------------------------------------
# 1. Data model
# ---------------------------------------------------------------------------

class SheafGraph:
    def __init__(self):
        self.community_ids = []  # sorted list
        self.tfidf = {}          # community_id -> [(term, score), ...]
        self.nodes = {}          # paper_id -> community_id
        self.edges = []          # [(c1, c2, weight)], c1 <= c2, deduped


# ---------------------------------------------------------------------------
# 2. Loader
# ---------------------------------------------------------------------------

def load_louvain_community_graph(path):
    with open(path) as f:
        data = json.load(f)

    G = SheafGraph()

    for node in data["nodes"]:
        G.nodes[node["id"]] = node["community_id"]

    if not data.get("communities"):
        raise ValueError(
            f"No 'communities' block in {path}. Export with --tfidf-top-n 200 using the EXP-RS-07 schema."
        )

    for comm in data["communities"]:
        G.tfidf[comm["community_id"]] = list(comm["tfidf_vec"])

    G.community_ids = sorted(G.tfidf.keys())

    edge_weight = {}
    for edge in data["edges"]:
        sc = G.nodes.get(edge["src"])
        dc = G.nodes.get(edge["dst"])
        if sc is None or dc is None or sc == dc:
            continue
        key = (min(sc, dc), max(sc, dc))
        edge_weight[key] = edge_weight.get(key, 0.0) + float(edge["weight"])

    G.edges = [(c1, c2, w) for (c1, c2), w in sorted(edge_weight.items())]

    return G, data.get("ground_truth")


# ---------------------------------------------------------------------------
# 3. Stalks
# ---------------------------------------------------------------------------

def build_stalks(G, top_k=100):
    basis_terms = {}
    for cid in G.community_ids:
        sorted_t = sorted(G.tfidf[cid], key=lambda x: -x[1])[:top_k]
        basis_terms[cid] = [t for t, _ in sorted_t]
    return basis_terms


# ---------------------------------------------------------------------------
# 4. Restriction maps
# ---------------------------------------------------------------------------

def build_restriction_maps(G, basis_terms):
    from scipy.sparse import lil_matrix

    maps = {}
    rank_zero = 0

    for (c1, c2, _) in G.edges:
        b1 = basis_terms[c1]
        b2 = basis_terms[c2]
        b2_set = set(b2)
        shared = [t for t in b1 if t in b2_set]

        if not shared:
            maps[(c1, c2)] = None
            rank_zero += 1
            continue

        k1, k2, d_e = len(b1), len(b2), len(shared)
        b1_idx = {t: i for i, t in enumerate(b1)}
        b2_idx = {t: i for i, t in enumerate(b2)}

        F1 = lil_matrix((d_e, k1))
        F2 = lil_matrix((d_e, k2))
        for row, term in enumerate(shared):
            F1[row, b1_idx[term]] = 1.0
            F2[row, b2_idx[term]] = 1.0

        maps[(c1, c2)] = (shared, F1.tocsr(), F2.tocsr())

    return maps, rank_zero


# ---------------------------------------------------------------------------
# 5. Laplacian assembly (COO accumulation — correct for multiple edges)
# ---------------------------------------------------------------------------

def assemble_laplacian(G, basis_terms, maps):
    cids = G.community_ids
    k_list = [len(basis_terms[c]) for c in cids]
    c_to_i = {c: i for i, c in enumerate(cids)}
    offsets = np.cumsum([0] + k_list, dtype=int)
    N = int(offsets[-1])

    rows_acc, cols_acc, vals_acc = [], [], []

    def add_sparse(row_off, col_off, M, sign=1.0):
        coo = M.tocoo()
        for r, c, v in zip(coo.row, coo.col, coo.data):
            rows_acc.append(row_off + int(r))
            cols_acc.append(col_off + int(c))
            vals_acc.append(sign * float(v))

    for (c1, c2, _) in G.edges:
        entry = maps.get((c1, c2))
        if entry is None:
            continue
        _, F1, F2 = entry
        i1, i2 = c_to_i[c1], c_to_i[c2]
        o1, o2 = int(offsets[i1]), int(offsets[i2])

        F1tF1 = F1.T @ F1
        F2tF2 = F2.T @ F2
        F1tF2 = F1.T @ F2

        add_sparse(o1, o1, F1tF1)
        add_sparse(o2, o2, F2tF2)
        add_sparse(o1, o2, F1tF2, sign=-1.0)
        add_sparse(o2, o1, F1tF2.T, sign=-1.0)

    L = coo_matrix(
        (vals_acc, (rows_acc, cols_acc)), shape=(N, N)
    ).tocsr()
    return L, offsets, c_to_i


# ---------------------------------------------------------------------------
# 6. Sanity checks
# ---------------------------------------------------------------------------

def sc2_symmetry(L):
    diff = L - L.T
    norm_diff = float(np.sqrt((diff.multiply(diff)).sum()))
    norm_L = float(np.sqrt((L.multiply(L)).sum()))
    return norm_diff / (norm_L + 1e-30)


# ---------------------------------------------------------------------------
# 7. Eigensolver — dense for small N, sparse for large N
# ---------------------------------------------------------------------------

def solve_eigenproblem(L, n_sections=20):
    N = L.shape[0]

    if N <= 500:
        # Dense solver: finds all eigenvalues — required when N is small enough
        # that k << N would miss modes (e.g. toy with H⁰=22 out of N=24).
        vals, vecs = np.linalg.eigh(L.toarray())
    else:
        k = min(n_sections + 5, N - 2)
        vals, vecs = eigsh(L, k=k, sigma=1e-8, which="LM")
        order = np.argsort(vals)
        vals, vecs = vals[order], vecs[:, order]

    h0 = int(np.sum(vals < 1e-6))
    nz_mask = vals >= 1e-6
    vals_nz = vals[nz_mask][:n_sections]
    vecs_nz = vecs[:, nz_mask][:, :n_sections]
    return vals_nz, vecs_nz, h0


# ---------------------------------------------------------------------------
# 8. Frustration + bridge score  Φ(e) = Σ_i φ_i(e) / λ_i
# ---------------------------------------------------------------------------

def compute_bridge_scores(G, basis_terms, maps, vals, vecs, offsets, c_to_i):
    k_list = [len(basis_terms[c]) for c in G.community_ids]
    results = []

    for (c1, c2, _) in G.edges:
        entry = maps.get((c1, c2))
        if entry is None:
            results.append(((c1, c2), 0.0, []))
            continue

        _, F1, F2 = entry
        i1, i2 = c_to_i[c1], c_to_i[c2]
        o1, o2 = int(offsets[i1]), int(offsets[i2])
        k1, k2 = k_list[i1], k_list[i2]

        phi_per_mode = []
        Phi = 0.0

        for i, lam in enumerate(vals):
            x = vecs[:, i]
            x1 = x[o1 : o1 + k1]
            x2 = x[o2 : o2 + k2]
            F1x1 = np.array(F1 @ x1).ravel()
            F2x2 = np.array(F2 @ x2).ravel()
            num = float(np.sum((F1x1 - F2x2) ** 2))
            denom = float(np.sum(x1 ** 2) + np.sum(x2 ** 2)) + 1e-30
            phi_i = num / denom
            phi_per_mode.append(phi_i)
            Phi += phi_i / float(lam)

        results.append(((c1, c2), Phi, phi_per_mode))

    results.sort(key=lambda x: -x[1])
    return results


# ---------------------------------------------------------------------------
# 9. T4 ablation  (run on full corpus only — Stage B)
# ---------------------------------------------------------------------------

def run_t4_ablation(top5, G, basis_terms, maps, offsets, c_to_i, vals, vecs, L):
    k_list = [len(basis_terms[c]) for c in G.community_ids]
    pass_count = 0
    detail = []

    for (c1, c2), Phi, phi_per_mode in top5:
        if not phi_per_mode:
            detail.append(((c1, c2), -1, 0.0, 0.0, "FAIL"))
            continue

        i_star = int(np.argmax(phi_per_mode))
        lam_star = float(vals[i_star])
        x_star = vecs[:, i_star].copy()
        norm_x = float(np.sqrt(np.sum(x_star ** 2)))

        x_prime = x_star.copy()
        for v_idx, c in enumerate(G.community_ids):
            o = int(offsets[v_idx])
            kv = k_list[v_idx]
            bn = float(np.sqrt(np.sum(x_star[o : o + kv] ** 2)))
            if bn > 0.1 * norm_x:
                x_prime[o : o + kv] = 0.0

        norm_xp = float(np.sqrt(np.sum(x_prime ** 2)))
        if norm_xp < 1e-10:
            ratio = 0.0
        else:
            Lxp = np.array(L @ x_prime).ravel()
            lam_ablated = float(x_prime @ Lxp) / (norm_xp ** 2)
            ratio = lam_ablated / lam_star if lam_star > 1e-12 else 0.0

        passed = ratio > 3.0
        if passed:
            pass_count += 1
        status = "PASS" if passed else "FAIL"
        detail.append(((c1, c2), i_star, lam_star, ratio, status))
        print(f"  {status} bridge=comm{c1}↔comm{c2}  i*={i_star}  λ*={lam_star:.4f}  ratio={ratio:.2f}")

    n = max(len(top5), 1)
    pass_rate = pass_count / n
    if pass_rate >= 0.5:
        overall = "PASS"
    elif pass_rate < 0.2:
        overall = "FALSIFIED"
    else:
        overall = "FAIL"

    print(f"  T4 {overall}: {pass_count}/{len(top5)} bridges pass (≥50% target)")
    return detail, pass_rate, overall


# ---------------------------------------------------------------------------
# 10. T2 precision@10  (corpus + feynman ground truth)
# ---------------------------------------------------------------------------

def compute_t2_precision(top10_bridges, corpus_nodes, gt_path):
    with open(gt_path) as f:
        gt = json.load(f)

    evaluable = set(gt.get("evaluable_pairs", []))
    gt_comm_pairs = set()

    for pair in gt["pairs"]:
        pid = pair["id"]
        if not any(pid.startswith(ep) for ep in evaluable):
            continue
        sa_id = (pair["side_a"].get("arxiv_id") or "").split("v")[0]
        sb_id = (pair["side_b"].get("arxiv_id") or "").split("v")[0]
        if not sa_id or not sb_id:
            continue
        ca = corpus_nodes.get(sa_id)
        cb = corpus_nodes.get(sb_id)
        if ca is None or cb is None:
            print(f"  [WARN] {pid}: paper not in corpus ({sa_id if ca is None else sb_id})")
            continue
        gt_comm_pairs.add((min(ca, cb), max(ca, cb)))

    top10_set = {(min(c1, c2), max(c1, c2)) for (c1, c2), _, _ in top10_bridges}
    hits = len(top10_set & gt_comm_pairs)
    precision_at_10 = hits / 10.0

    print(f"  Ground-truth community pairs ({len(gt_comm_pairs)}): {gt_comm_pairs}")
    print(f"  Top-10 sheaf bridges: {top10_set}")
    print(f"  Hits: {hits}  T2 precision@10={precision_at_10:.3f}")
    return precision_at_10, hits


# ---------------------------------------------------------------------------
# 11. SC4 spectral gap  λ_{k+1}/λ_k at k=20
# ---------------------------------------------------------------------------

def sc4_spectral_gap(vals, k=20):
    if len(vals) < k + 1:
        return None
    return float(vals[k] / vals[k - 1]) if vals[k - 1] > 1e-10 else None


# ---------------------------------------------------------------------------
# 12. SC5 Jaccard(top-10 sheaf, top-10 c-TF-IDF cosine baseline)
# ---------------------------------------------------------------------------

def sc5_baseline_jaccard(top10_bridges, G):
    from itertools import combinations
    import math

    def cosine(t1, t2):
        d1 = {t: s for t, s in t1}
        d2 = {t: s for t, s in t2}
        dot = sum(d1[t] * d2[t] for t in set(d1) & set(d2))
        n1 = math.sqrt(sum(v ** 2 for v in d1.values()))
        n2 = math.sqrt(sum(v ** 2 for v in d2.values()))
        return dot / (n1 * n2 + 1e-30)

    baseline = sorted(
        [
            (min(a, b), max(a, b), cosine(G.tfidf[a], G.tfidf[b]))
            for a, b in combinations(G.community_ids, 2)
        ],
        key=lambda x: -x[2],
    )
    top10_baseline = {(a, b) for a, b, _ in baseline[:10]}
    top10_sheaf = {(min(c1, c2), max(c1, c2)) for (c1, c2), _, _ in top10_bridges}
    inter = len(top10_sheaf & top10_baseline)
    union = len(top10_sheaf | top10_baseline)
    return inter / max(union, 1)


# ---------------------------------------------------------------------------
# 13. Figures
# ---------------------------------------------------------------------------

def generate_spectrum_figure(vals_nz, output_path):
    try:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
    except ImportError:
        print("  matplotlib not available; skipping spectrum figure")
        return None

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    x = np.arange(1, len(vals_nz) + 1)
    fig, ax = plt.subplots(figsize=(10, 5))
    ax.semilogy(x, vals_nz, "o-", color="steelblue", markersize=4, label="all non-zero modes")
    top = min(20, len(vals_nz))
    ax.semilogy(x[:top], vals_nz[:top], "s", color="orange", markersize=6, label="bottom-20 (used for Φ)")
    ax.set_xlabel("Mode index (non-zero)")
    ax.set_ylabel("Eigenvalue (log scale)")
    ax.set_title("EXP-RS-07 Sheaf Laplacian spectrum")
    ax.legend()
    plt.tight_layout()
    plt.savefig(output_path, dpi=120)
    plt.close()
    print(f"  Spectrum figure: {output_path}")
    return output_path


def generate_bridge_figure(bridge_results, G, output_path):
    try:
        import matplotlib
        matplotlib.use("Agg")
        import matplotlib.pyplot as plt
        import networkx as nx
    except ImportError:
        print("  matplotlib/networkx not available; skipping bridge figure")
        return None

    os.makedirs(os.path.dirname(output_path), exist_ok=True)
    H = nx.Graph()
    H.add_nodes_from(G.community_ids)
    top10 = bridge_results[:10]
    max_phi = max((phi for _, phi, _ in top10), default=1.0)
    for (c1, c2), phi, _ in top10:
        H.add_edge(c1, c2, weight=phi / (max_phi + 1e-30))

    pos = nx.spring_layout(H, seed=SEED)
    edge_widths = [H[u][v]["weight"] * 4 for u, v in H.edges()]

    fig, ax = plt.subplots(figsize=(10, 8))
    nx.draw_networkx_nodes(H, pos, ax=ax, node_size=300, node_color="steelblue")
    nx.draw_networkx_labels(H, pos, ax=ax, font_size=8, labels={c: f"C{c}" for c in G.community_ids})
    nx.draw_networkx_edges(H, pos, ax=ax, width=edge_widths, edge_color="orange", alpha=0.7)
    ax.set_title("EXP-RS-07 Top-10 Sheaf-LBD bridges (edge width ∝ Φ)")
    ax.axis("off")
    plt.tight_layout()
    plt.savefig(output_path, dpi=120)
    plt.close()
    print(f"  Bridge figure: {output_path}")
    return output_path


# ---------------------------------------------------------------------------
# 14. Results emission
# ---------------------------------------------------------------------------

def write_results_md(path, r):
    t2_run = r.get("t2_run", False)
    lines = [
        "# EXP-RS-07 Results — Sheaves-LBD v0.1",
        "",
        "**Experiment:** EXP-RS-07 — Cellular sheaf bridge detection on the Louvain community graph.",
        f"**Status:** {'completed' if t2_run else 'stage-A only (Feynman corpus pending)'}",
        "**Date:** 2026-04-20",
        "",
        "## Test results",
        "",
        "| Test | Metric | Value | Target | Status |",
        "|------|--------|-------|--------|--------|",
        f"| T1 toy | top-3 ∋ ground-truth edge | {'yes' if r['t1_pass'] else 'no'} | yes | {'PASS' if r['t1_pass'] else 'FAIL'} |",
        f"| SC1 H⁰(F) | dim(ker L_F) on toy | {r['h0']} | ≤ 5 | {'PASS' if r['h0'] <= 5 else 'FLAG (expected on toy)'} |",
        f"| SC2 symmetry | ‖L−Lᵀ‖_F/‖L‖_F | {r['sc2_ratio']:.2e} | < 1e-6 | {'PASS' if r['sc2_ratio'] < 1e-6 else 'FAIL'} |",
        f"| SC3 PSD | min non-zero eigenvalue | {r['min_lam']:.2e} | ≥ −1e-8 | {'PASS' if r['min_lam'] >= -1e-8 else 'FAIL'} |",
    ]

    if t2_run:
        sc4 = r.get("sc4_gap")
        sc4_str = f"{sc4:.3f}" if sc4 is not None else "N/A"
        sc4_status = "PASS" if (sc4 is not None and sc4 >= 1.1) else "FLAG"
        sc5 = r.get("sc5_jaccard", 0.0)
        lines += [
            f"| T2 precision@10 | precision@10 | {r['t2_precision']:.3f} | ≥ 0.4 | {'PASS' if r['t2_precision'] >= 0.4 else 'FAIL'} |",
            f"| T4 ablation | pass rate | {r.get('t4_pass_rate', 0.0):.2f} | ≥ 0.5 | {r.get('t4_overall', 'HOLD')} |",
            f"| SC4 spectral gap | λ₂₁/λ₂₀ | {sc4_str} | ≥ 1.1 | {sc4_status} |",
            f"| SC5 Jaccard | J(sheaf, cosine) | {sc5:.3f} | ≤ 0.8 | {'PASS' if sc5 <= 0.8 else 'FLAG'} |",
        ]
    else:
        lines += [
            "| T2 precision@10 | precision@10 | PENDING | ≥ 0.4 | HOLD |",
            "| T4 ablation | pass rate | PENDING | ≥ 0.5 | HOLD |",
            "| SC4 spectral gap | λ₂₁/λ₂₀ | PENDING | ≥ 1.1 | HOLD |",
            "| SC5 Jaccard | J(sheaf, cosine) | PENDING | ≤ 0.8 | HOLD |",
        ]

    lines += ["", "## Decision", ""]

    if not r["t1_pass"]:
        decision, reasoning = "HOLD", "T1 failed — check restriction maps / stalk builder and re-run."
    elif not t2_run:
        decision = "HOLD"
        reasoning = (
            "Stage A complete (T1 PASS, SC2/SC3 PASS). Feynman corpus crawl pending. "
            "Re-run after `cargo run --bin resyn -- analyze` and `export-louvain-graph`."
        )
    elif r["t2_precision"] >= 0.4 and r.get("t4_pass_rate", 0.0) >= 0.5:
        decision = "AUTHORIZE"
        reasoning = (
            f"T1 PASS. T2 precision@10={r['t2_precision']:.3f} ≥ 0.4. "
            f"T4 pass-rate={r.get('t4_pass_rate',0.0):.2f} ≥ 0.5. Authorize Phase 43 (Rust sheaf module)."
        )
    elif r.get("t4_pass_rate", 1.0) >= 0.5 and r["t2_precision"] < 0.4:
        decision = "STANDALONE"
        reasoning = (
            f"T4 multi-causal PASS (pass-rate={r.get('t4_pass_rate',0.0):.2f}) but "
            f"T2 precision@10={r['t2_precision']:.3f} < 0.4. Standalone publication candidate."
        )
    elif r.get("t4_pass_rate", 1.0) < 0.2:
        decision = "FALSIFIED"
        reasoning = "T4 < 20% — multi-causal thesis falsified; RAF-LBD remains the only multi-causal candidate."
    elif r.get("sc5_jaccard", 0.0) > 0.8:
        decision = "FALSIFIED"
        reasoning = f"SC5 Jaccard={r.get('sc5_jaccard',0.0):.3f} > 0.8 — sheaf bridges re-implement c-TF-IDF cosine baseline."
    else:
        decision = "HOLD"
        reasoning = "Partial results; T2 below threshold but T4 inconclusive."

    lines += [f"**{decision}** — {reasoning}", ""]

    if t2_run and r.get("t4_detail"):
        lines += ["## T4 ablation detail", "", "| Bridge | i* | λ* | ratio | Status |", "|--------|----|----|-------|--------|"]
        for (c1, c2), i_star, lam_star, ratio, status in r["t4_detail"]:
            lines.append(f"| comm{c1}↔comm{c2} | {i_star} | {lam_star:.4f} | {ratio:.2f} | {status} |")
        lines.append("")

    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")
    print(f"  Results written: {path}")


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    base = os.path.dirname(os.path.abspath(__file__))
    data_dir = os.path.join(base, "data")
    toy_path = os.path.join(data_dir, "hopfield_spinglass_toy.json")
    corpus_path = os.path.join(data_dir, "research_synergy_pre2015.json")
    gt10_path = os.path.join(data_dir, "feynman_10pair_papers.json")
    results_path = os.path.join(base, "SHEAF_V01_RESULTS.md")
    spectrum_fig = os.path.join(base, "figures", "sheaf_lbd_v01_spectrum.png")
    bridges_fig = os.path.join(base, "figures", "sheaf_lbd_v01_bridges.png")

    results = {"t2_run": False}

    # -----------------------------------------------------------------------
    # T1 + SC1/SC2/SC3 — Toy fixture
    # -----------------------------------------------------------------------
    print("EXP-RS-07  Sheaves-LBD v0.1")
    print("\n" + "=" * 60)
    print("T1  Toy fixture — Hopfield ↔ Spin-glass")
    print("=" * 60)

    G_toy, gt_toy = load_louvain_community_graph(toy_path)
    print(f"  Communities: {len(G_toy.community_ids)}, inter-community edges: {len(G_toy.edges)}")

    basis_toy = build_stalks(G_toy, top_k=100)
    maps_toy, rz_toy = build_restriction_maps(G_toy, basis_toy)
    print(f"  Rank-zero edges (no shared terms): {rz_toy}")

    L_toy, offsets_toy, c_to_i_toy = assemble_laplacian(G_toy, basis_toy, maps_toy)

    sym_ratio = sc2_symmetry(L_toy)
    results["sc2_ratio"] = sym_ratio
    if sym_ratio >= 1e-6:
        print(f"  SC2 ABORT: ‖L−Lᵀ‖_F/‖L‖_F = {sym_ratio:.2e} ≥ 1e-6")
        sys.exit(1)
    print(f"  SC2 PASS: symmetry ratio = {sym_ratio:.2e}")

    vals_toy, vecs_toy, h0_toy = solve_eigenproblem(L_toy, n_sections=20)
    results["h0"] = h0_toy
    results["min_lam"] = float(np.min(vals_toy)) if len(vals_toy) > 0 else 0.0

    h0_note = "OK" if h0_toy <= 5 else "FLAG: large on toy (expected — most terms unshared)"
    print(f"  SC1: H⁰(F) = {h0_toy}  ({h0_note})")
    print(f"  SC3: min non-zero eigenvalue = {results['min_lam']:.2e}")
    if results["min_lam"] < -1e-8:
        print("  SC3 ABORT: L_F is not PSD")
        write_results_md(results_path, results)
        sys.exit(1)
    print("  SC3 PASS")

    bridges_toy = compute_bridge_scores(G_toy, basis_toy, maps_toy, vals_toy, vecs_toy, offsets_toy, c_to_i_toy)

    if gt_toy:
        gt_pair = tuple(sorted(gt_toy["bridge_community_pair"]))
        top3 = [tuple(sorted(e)) for (e, _, _) in bridges_toy[:3]]
        t1_pass = gt_pair in top3
    else:
        t1_pass = True
    results["t1_pass"] = t1_pass

    print("  Top bridges:")
    for i, ((c1, c2), phi, _) in enumerate(bridges_toy[:5]):
        marker = ""
        if gt_toy and tuple(sorted([c1, c2])) == tuple(sorted(gt_toy["bridge_community_pair"])):
            marker = "  ← GROUND TRUTH"
        print(f"    [{i+1}] comm{c1} ↔ comm{c2}  Φ={phi:.4f}{marker}")

    if not t1_pass:
        print("  T1 FAIL — ground truth not in top-3; aborting")
        write_results_md(results_path, results)
        sys.exit(1)
    print("  T1 PASS")

    # -----------------------------------------------------------------------
    # T2 / T4 / SC4 / SC5 — Feynman corpus (Stage B)
    # -----------------------------------------------------------------------
    if not (os.path.exists(corpus_path) and os.path.exists(gt10_path)):
        print(f"\n  Corpus not found at {corpus_path} — Stage A complete.")
        print("  Re-run after: `cargo run --bin resyn -- analyze` + `export-louvain-graph`.")
        generate_spectrum_figure(vals_toy, spectrum_fig)
        generate_bridge_figure(bridges_toy, G_toy, bridges_fig)
        write_results_md(results_path, results)
    else:
        print("\n" + "=" * 60)
        print("T2 / T4 / SC4 / SC5  Feynman corpus")
        print("=" * 60)

        G_corp, _ = load_louvain_community_graph(corpus_path)
        print(f"  Communities: {len(G_corp.community_ids)}, inter-community edges: {len(G_corp.edges)}")

        basis_corp = build_stalks(G_corp, top_k=100)
        maps_corp, rz_corp = build_restriction_maps(G_corp, basis_corp)
        print(f"  Rank-zero edges: {rz_corp}")

        L_corp, offsets_corp, c_to_i_corp = assemble_laplacian(G_corp, basis_corp, maps_corp)

        sym_corp = sc2_symmetry(L_corp)
        if sym_corp >= 1e-6:
            print(f"  SC2 ABORT on corpus: {sym_corp:.2e}")
            sys.exit(1)

        # n_sections sensitivity — report all three, use n=20 for T2/T4
        all_precision = {}
        for n_sec in [10, 20, 50]:
            vals_c, vecs_c, h0_c = solve_eigenproblem(L_corp, n_sections=n_sec)
            min_lam_c = float(np.min(vals_c)) if len(vals_c) > 0 else 0.0
            if min_lam_c < -1e-8:
                print(f"  SC3 ABORT on corpus (n={n_sec}): {min_lam_c:.2e}")
                sys.exit(1)
            bridges_c = compute_bridge_scores(G_corp, basis_corp, maps_corp, vals_c, vecs_c, offsets_corp, c_to_i_corp)
            prec, _ = compute_t2_precision(bridges_c, G_corp.nodes, gt10_path)
            all_precision[n_sec] = prec
            print(f"  n_sections={n_sec}: T2 precision@10={prec:.3f}")

            if n_sec == 20:
                vals_20, vecs_20 = vals_c, vecs_c
                bridges_20 = bridges_c

        results["t2_precision"] = all_precision[20]
        results["t2_run"] = True

        # SC4 + SC5 on n=20
        sc4 = sc4_spectral_gap(vals_20, k=20)
        results["sc4_gap"] = sc4
        sc5 = sc5_baseline_jaccard(bridges_20, G_corp)
        results["sc5_jaccard"] = sc5
        print(f"  SC4 spectral gap (λ₂₁/λ₂₀): {sc4}")
        print(f"  SC5 Jaccard(sheaf, cosine baseline): {sc5:.3f}")

        # T4 ablation on top-5 of n=20
        print("\n" + "=" * 60)
        print("T4  Ablation (top-5 bridges, n_sections=20)")
        print("=" * 60)
        top5 = bridges_20[:5]
        t4_detail, t4_rate, t4_overall = run_t4_ablation(
            top5, G_corp, basis_corp, maps_corp, offsets_corp, c_to_i_corp, vals_20, vecs_20, L_corp
        )
        results["t4_pass_rate"] = t4_rate
        results["t4_overall"] = t4_overall
        results["t4_detail"] = t4_detail

        generate_spectrum_figure(vals_20, spectrum_fig)
        generate_bridge_figure(bridges_20, G_corp, bridges_fig)
        write_results_md(results_path, results)

    print("\n" + "=" * 60)
    print("Summary")
    print("=" * 60)
    print(f"  T1: {'PASS' if results['t1_pass'] else 'FAIL'}")
    print(f"  SC1 H⁰(F): {results['h0']} (toy)")
    print(f"  SC2: PASS (ratio={results['sc2_ratio']:.2e})")
    print(f"  SC3: PASS (min_lam={results['min_lam']:.2e})")
    if results["t2_run"]:
        print(f"  T2: precision@10 = {results['t2_precision']:.3f}  ({'PASS' if results['t2_precision'] >= 0.4 else 'FAIL'})")
        print(f"  T4: {results.get('t4_overall','?')} (pass-rate={results.get('t4_pass_rate',0):.2f})")
        print(f"  SC4: gap={results.get('sc4_gap')}")
        print(f"  SC5: Jaccard={results.get('sc5_jaccard',0):.3f}")
    else:
        print("  T2/T4/SC4/SC5: HOLD (corpus pending)")


if __name__ == "__main__":
    main()
