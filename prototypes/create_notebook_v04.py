#!/usr/bin/env python3
"""Generate kuramoto_lbd_v04.ipynb — EXP-RS-11 TF-IDF semantic-edge substrate.

Adapted from kuramoto_lbd_v03.ipynb with exactly the pre-registered changes
(vault: wiki/meta/agentic-experiments-research.md § EXP-RS-11):
  1. TF-IDF cosine adjacency (from the committed build_tfidf_graph.py artifact)
     replaces the citation adjacency.
  2. Connectivity precheck (n_cc/N <= 0.05) before compute_K_stable; dynamics
     run only at thresholds that pass.
  3. compute_K_stable gets a 300 s wall-clock budget.
All v03 pipeline code (Kuramoto ODE, sync-Laplacian Fiedler bridges, SC gates,
nulls, plateau, Feynman benchmark) is carried over unchanged so the notebook
runs to a real BENCH_P10 whenever the precheck passes.
"""

import nbformat as nbf

nb = nbf.v4.new_notebook()
cells = []


def md(src):
    cells.append(nbf.v4.new_markdown_cell(src))


def code(src):
    cells.append(nbf.v4.new_code_cell(src))


md("""# Kuramoto-LBD Prototype v0.4 — TF-IDF Semantic-Edge Substrate (EXP-RS-11)

**Changes vs v0.3 (pre-registered, vault § EXP-RS-11):**
1. Adjacency = TF-IDF cosine semantic edges from `data/tfidf_graph_pre2015.json`
   (built by the committed `build_tfidf_graph.py`; edge iff cosine ≥ τ, weight = cosine),
   replacing the citation adjacency.
2. **Connectivity precheck** before `compute_K_stable`: dynamics run only at
   τ ∈ {0.2, 0.3, 0.4, 0.5} where `n_cc/N ≤ 0.05`. If no τ passes, all dynamics
   cells SKIP and the summary reports the kill-gate outcome.
3. `compute_K_stable` has a **300 s wall-clock budget** (locked prediction P-3).

Locked predictions (do not adjust post-hoc):
- P-1: at τ=0.3, `n_cc/N ≤ 0.05` (citation graph: 0.27)
- P-2: at τ=0.3, largest CC ≥ 80% of nodes (citation graph: 38%)
- P-3: `compute_K_stable` completes within the 5-minute budget
- P-4: benchmark gate `n_eval ≥ 3` AND `BENCH_P10 > 0.15` (≥ 0.30 = success, EXP-RS-06)
""")

code("""import hashlib
import json
import os
import time
import numpy as np
from scipy.integrate import solve_ivp
from scipy.sparse import csr_matrix, diags, lil_matrix
from scipy.sparse.csgraph import connected_components as sp_connected_components
from scipy.sparse.linalg import eigsh
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

# --- Paths ---
DATA_FILE        = 'data/research_synergy_pre2015.json'
TFIDF_GRAPH_FILE = 'data/tfidf_graph_pre2015.json'   # committed build_tfidf_graph.py artifact
FEYNMAN_FILE     = 'data/feynman_10pair_papers.json'
ABC_BRIDGES      = 'data/abc_bridges.json'   # optional; skip check if absent

# --- Protocol constants (pre-registered, do not change post-hoc) ---
GLOBAL_SEED    = 42
K_MULTS_V03    = [1.3, 1.5, 2.0, 3.0]  # restricted to synchronisation regime (v03 convention)
MID_K          = 1.3                    # plateau representative

PRECISION_AT_K         = 10
PRECISION_TARGET       = 0.30
TFIDF_BASELINE_EXPECT  = 0.15
PLATEAU_JAC_MIN        = 0.85
NULL_INFLATE_ABORT     = 0.25
ORTHO_RETIRE_JAC       = 0.70

# --- v04 additions (pre-registered, EXP-RS-11 / conventions C-6) ---
TAU_SWEEP              = [0.2, 0.3, 0.4, 0.5]
TAU_HEADLINE           = 0.3
PRECHECK_MAX_NCC_FRAC  = 0.05
KSTABLE_BUDGET_S       = 300.0

print('Imports OK.')
print(f'Seeds: GLOBAL_SEED={GLOBAL_SEED} (omega, theta0), ER null=99, CM null=77')
print(f'K_mults: {K_MULTS_V03}')
print(f'Pre-registered precision@{PRECISION_AT_K} target: {PRECISION_TARGET}')
print(f'tau sweep: {TAU_SWEEP}, headline tau={TAU_HEADLINE}, precheck n_cc/N <= {PRECHECK_MAX_NCC_FRAC}')
print(f'K_stable budget: {KSTABLE_BUDGET_S:.0f} s')""")

code("""# --- Load Louvain corpus JSON + committed TF-IDF graph artifact ---
for path in (DATA_FILE, TFIDF_GRAPH_FILE):
    if not os.path.exists(path):
        raise FileNotFoundError(f'Missing input: {path}')

with open(DATA_FILE) as f:
    corpus = json.load(f)
with open(TFIDF_GRAPH_FILE) as f:
    tfidf_graph = json.load(f)

# Integrity: the TF-IDF graph artifact must have been built from THIS export.
with open(DATA_FILE, 'rb') as f:
    export_sha = hashlib.sha256(f.read()).hexdigest()
assert tfidf_graph['input_sha256'] == export_sha, (
    f'TF-IDF artifact was built from a different export: {tfidf_graph["input_sha256"]} != {export_sha}')
print(f'Integrity OK: artifact input_sha256 == sha256({DATA_FILE})')
print(f'Corpus fingerprint: {corpus.get("corpus_fingerprint", "N/A")}')

nodes         = corpus['nodes']
N             = len(nodes)
node_idx      = {n['id']: i for i, n in enumerate(nodes)}
community_ids = np.array([n['community_id'] for n in nodes], dtype=np.int64)
n_communities = int(np.unique(community_ids).size)
print(f'Corpus: N={N} nodes, {n_communities} communities')

# --- Build symmetric TF-IDF cosine adjacency per tau (weight = cosine) ---
A_by_tau = {}
sweep_recomputed = []
for tau in TAU_SWEEP:
    key = str(tau)
    edges = tfidf_graph['graphs'][key]['edges']
    rows, cols, vals = [], [], []
    for e in edges:
        i, j, w = node_idx[e['src']], node_idx[e['dst']], float(e['weight'])
        rows += [i, j]; cols += [j, i]; vals += [w, w]
    A_tau = csr_matrix((vals, (rows, cols)), shape=(N, N))
    # Independent (third) connectivity computation: scipy.sparse.csgraph
    n_cc, labels = sp_connected_components(A_tau, directed=False)
    sizes = np.bincount(labels)
    largest = int(sizes.max()) if len(sizes) else 0
    deg = np.array((A_tau != 0).sum(axis=1)).flatten()
    row = {'tau': tau, 'n_edges': A_tau.nnz // 2, 'n_cc': int(n_cc),
           'largest_cc_size': largest, 'ncc_frac': n_cc / N,
           'n_isolated': int((deg == 0).sum())}
    sweep_recomputed.append(row)
    A_by_tau[tau] = A_tau

# Cross-check against the committed artifact's sweep stats.
print()
print(f'{"tau":>5} {"n_edges":>8} {"n_cc":>5} {"largest":>8} {"ncc/N":>7} {"precheck":>9}  artifact-match')
for row in sweep_recomputed:
    art = next(r for r in tfidf_graph['sweep'] if abs(r['tau'] - row['tau']) < 1e-9)
    match = (art['n_edges'] == row['n_edges'] and art['n_cc'] == row['n_cc']
             and art['largest_cc_size'] == row['largest_cc_size'])
    ok = row['ncc_frac'] <= PRECHECK_MAX_NCC_FRAC
    print(f'{row["tau"]:>5.2f} {row["n_edges"]:>8d} {row["n_cc"]:>5d} {row["largest_cc_size"]:>8d} '
          f'{row["ncc_frac"]:>7.3f} {"PASS" if ok else "FAIL":>9}  {"OK" if match else "MISMATCH"}')
    assert match, f'scipy connected_components disagrees with build_tfidf_graph.py at tau={row["tau"]}'

TAU_PASSING  = [r['tau'] for r in sweep_recomputed if r['ncc_frac'] <= PRECHECK_MAX_NCC_FRAC]
RUN_DYNAMICS = len(TAU_PASSING) > 0
TAU_HEAD     = TAU_HEADLINE if TAU_HEADLINE in TAU_PASSING else (TAU_PASSING[0] if TAU_PASSING else None)

print()
if RUN_DYNAMICS:
    print(f'PRECHECK: PASS at tau={TAU_PASSING}; dynamics will run at tau={TAU_HEAD}')
else:
    print('PRECHECK: FAIL at ALL tau — dynamics cells will SKIP.')
    print('Per EXP-RS-11 follow-ups: corpus too narrow for TF-IDF semantic edges at')
    print('pre-registered thresholds; pivot kill gate fires (decision -> human via vault).')""")

code("""# SC4 TOY GATE: run Fiedler extraction on a 50-node two-community toy graph.
# ABORT if true bridge not in top-20. This gates the pipeline correctness
# before the expensive corpus run. (Unchanged from v03; runs regardless of precheck.)

def build_two_community_toy(n_per=25, p_intra=0.35, seed=42):
    rng = np.random.default_rng(seed)
    n   = 2 * n_per
    rows, cols = [], []
    for start in [0, n_per]:
        for i in range(start, start + n_per):
            for j in range(i + 1, start + n_per):
                if rng.random() < p_intra:
                    rows += [i, j]; cols += [j, i]
    bi, bj = n_per - 1, n_per
    rows += [bi, bj]; cols += [bj, bi]
    A_toy = csr_matrix((np.ones(len(rows)), (rows, cols)), shape=(n, n))
    return A_toy, (bi, bj)

def kuramoto_rhs(t, theta, K, omega, A_csr, D_inv_arr):
    cos_t, sin_t = np.cos(theta), np.sin(theta)
    coupling     = (A_csr @ sin_t) * cos_t - (A_csr @ cos_t) * sin_t
    return omega + K * D_inv_arr * coupling

def build_sync_laplacian(theta_star, A, K):
    i_idx, j_idx = A.nonzero()
    cos_diffs    = np.cos(theta_star[j_idx] - theta_star[i_idx])
    n            = A.shape[0]
    J_off        = csr_matrix((K * cos_diffs, (i_idx, j_idx)), shape=(n, n))
    J_diag       = np.array(J_off.sum(axis=1)).flatten()
    return -J_off + diags(J_diag)

def extract_fiedler_bridges_sparse(L_sync, A, k_eigs=5, top_k=100, dense_fallback=False):
    \"\"\"Fiedler bridge extraction using sparse eigsh (shift-invert) or dense fallback.\"\"\"
    n = L_sync.shape[0]
    k = min(k_eigs, n - 2)
    if dense_fallback or n <= 500:
        ev, evec = np.linalg.eigh(L_sync.toarray())
        eigvals, eigvecs = ev[:k], evec[:, :k]
    else:
        try:
            eigvals, eigvecs = eigsh(L_sync, k=k, sigma=1e-8, which='LM', tol=1e-5)
        except Exception as err1:
            try:
                eigvals, eigvecs = eigsh(L_sync, k=k, which='SM', tol=1e-5)
            except Exception as err2:
                print(f'  [warn] sparse eigsh failed ({err2}), using dense fallback')
                ev, evec   = np.linalg.eigh(L_sync.toarray())
                eigvals, eigvecs = ev[:k], evec[:, :k]
    sort_idx = np.argsort(eigvals)
    eigvals  = eigvals[sort_idx]
    eigvecs  = eigvecs[:, sort_idx]
    v2       = eigvecs[:, 1]
    i_idx, j_idx = A.nonzero()
    upper    = i_idx < j_idx
    i_u, j_u = i_idx[upper], j_idx[upper]
    signs    = np.where(v2 >= 0, 1, -1)
    bridge_mask  = signs[i_u] != signs[j_u]
    contributions = np.abs(v2[i_u] * v2[j_u])
    bridge_edges  = list(zip(i_u[bridge_mask].tolist(), j_u[bridge_mask].tolist(),
                             contributions[bridge_mask].tolist()))
    bridge_edges.sort(key=lambda e: -e[2])
    return bridge_edges[:top_k], eigvals

def global_order_parameter(theta):
    return float(np.abs(np.mean(np.exp(1j * theta))))

A_toy, true_bridge = build_two_community_toy(n_per=25, p_intra=0.35, seed=42)
N_toy = A_toy.shape[0]
deg_toy   = np.array(A_toy.sum(axis=1)).flatten()
D_inv_toy = 1.0 / np.maximum(deg_toy, 1e-12)
rng_toy   = np.random.default_rng(GLOBAL_SEED)
omega_toy = rng_toy.normal(0.0, 1.0, size=N_toy); omega_toy -= omega_toy.mean()
theta0_toy = np.random.default_rng(GLOBAL_SEED).uniform(0, 2*np.pi, size=N_toy)

K_toy = 1.3 * 16.0  # approximate K_stable for toy graph
sol_toy = solve_ivp(kuramoto_rhs, (0, 200), theta0_toy.copy(),
                    args=(K_toy, omega_toy, A_toy, D_inv_toy),
                    method='RK45', rtol=1e-6, atol=1e-8, max_step=0.05)
theta_star_toy = sol_toy.y[:, -1]
L_sync_toy     = build_sync_laplacian(theta_star_toy, A_toy, K_toy)
bridges_toy, eigvals_toy = extract_fiedler_bridges_sparse(L_sync_toy, A_toy, k_eigs=5)

bi, bj = true_bridge
bridge_set_toy = [(int(e[0]), int(e[1])) for e in bridges_toy]
rank_toy       = (bridge_set_toy.index(true_bridge) + 1) if true_bridge in bridge_set_toy else None

print(f'SC4 TOY: true bridge {true_bridge} rank = {rank_toy}')
print(f'  Top-5: {bridge_set_toy[:5]}')
if rank_toy is not None and rank_toy <= 5:
    SC4_TOY = 'PASS'
elif rank_toy is not None and rank_toy <= 20:
    SC4_TOY = f'FLAG (rank={rank_toy})'
else:
    SC4_TOY = 'ABORT'
print(f'SC4 TOY verdict: {SC4_TOY}')
assert 'ABORT' not in SC4_TOY, f'Pipeline bug: toy bridge not in top-20. Debug extract_fiedler_bridges_sparse.'""")

code("""def _probe_lambda2(K, A_csr, omega, theta0, D_inv_arr, t_end=200, rtol=1e-5, atol=1e-7, max_step=0.5):
    \"\"\"Integrate Kuramoto ODE; return lambda_2(L_sync) and theta_star.\"\"\"
    sol        = solve_ivp(kuramoto_rhs, (0, t_end), theta0.copy(),
                           args=(K, omega, A_csr, D_inv_arr),
                           method='RK45', rtol=rtol, atol=atol,
                           dense_output=False, max_step=max_step)
    theta_star = sol.y[:, -1]
    L_sync     = build_sync_laplacian(theta_star, A_csr, K)
    n          = A_csr.shape[0]
    k          = min(3, n - 2)
    try:
        ev = eigsh(L_sync, k=k, sigma=1e-8, which='LM', tol=1e-4,
                   return_eigenvectors=False)
    except Exception:
        ev = np.linalg.eigh(L_sync.toarray())[0][:k]
    return float(np.sort(ev)[1]), theta_star


class KStableBudgetExceeded(RuntimeError):
    pass


def compute_K_stable(A_csr, omega, theta0, K_lo=0.5, K_hi=None,
                     tol=0.1, max_iter=15, budget_s=KSTABLE_BUDGET_S):
    \"\"\"Bisect for minimum K where lambda_2(L_sync(theta*(K), K)) >= 0.

    v04: enforces a wall-clock budget (pre-registered: 5 minutes). Every probe
    checks elapsed time first and raises KStableBudgetExceeded when over budget.
    \"\"\"
    t_start = time.time()

    def probe(K):
        if time.time() - t_start > budget_s:
            raise KStableBudgetExceeded(
                f'compute_K_stable exceeded {budget_s:.0f}s budget (locked prediction P-3 fails)')
        return _probe_lambda2(K, A_csr, omega, theta0, D_inv_arr)

    n          = A_csr.shape[0]
    deg_arr    = np.array(A_csr.sum(axis=1)).flatten()
    D_inv_arr  = 1.0 / np.maximum(deg_arr, 1e-12)
    if K_hi is None:
        L_uw = diags(deg_arr) - A_csr
        k_sp = min(3, n - 2)
        try:
            lam2_uw = float(np.sort(eigsh(L_uw, k=k_sp, which='SM',
                                          return_eigenvectors=False))[1])
        except Exception:
            lam2_uw = float(np.sort(np.linalg.eigh(L_uw.toarray())[0])[1])
        K_hi = max(4.0 / max(lam2_uw, 1e-6), 10.0)
        print(f'  lambda_2(L_unweighted) = {lam2_uw:.5f} -> K_hi init = {K_hi:.3f}')

    lam2_lo, _ = probe(K_lo)
    lam2_hi, _ = probe(K_hi)
    for _ in range(5):
        if lam2_lo < 0: break
        K_lo *= 0.5
        lam2_lo, _ = probe(K_lo)
    for _ in range(5):
        if lam2_hi >= 0: break
        K_hi *= 2.0
        lam2_hi, _ = probe(K_hi)
    if lam2_lo >= 0 or lam2_hi < 0:
        raise ValueError(f'Bisection bracket degenerate: lam2({K_lo})={lam2_lo:.4f}, lam2({K_hi})={lam2_hi:.4f}')
    trace = [(K_lo, lam2_lo), (K_hi, lam2_hi)]
    for _ in range(max_iter):
        if K_hi - K_lo < tol: break
        K_mid = (K_lo + K_hi) / 2
        lam2_mid, _ = probe(K_mid)
        trace.append((K_mid, lam2_mid))
        if lam2_mid < 0: K_lo, lam2_lo = K_mid, lam2_mid
        else:             K_hi, lam2_hi = K_mid, lam2_hi
    return K_hi, trace


def run_pipeline_sparse(A_csr, omega, K_ref, K_mults, theta0,
                        n_communities, top_k=100,
                        t_end=200, rtol=1e-5, atol=1e-7, max_step=0.5):
    \"\"\"K-sweep on sparse graph. Returns bridges_by_K, theta_by_K, eigvals_by_K.\"\"\"
    n         = A_csr.shape[0]
    deg_arr   = np.array(A_csr.sum(axis=1)).flatten()
    D_inv_arr = 1.0 / np.maximum(deg_arr, 1e-12)
    k_eigs    = min(n_communities + 2, n - 2)
    bridges_by_K, theta_by_K, eigvals_by_K = {}, {}, {}
    for K_mult in K_mults:
        K = K_mult * K_ref
        t0 = time.time()
        sol = solve_ivp(kuramoto_rhs, (0, t_end), theta0.copy(),
                        args=(K, omega, A_csr, D_inv_arr),
                        method='RK45', rtol=rtol, atol=atol,
                        dense_output=False, max_step=max_step)
        theta_star  = sol.y[:, -1]
        L_sync      = build_sync_laplacian(theta_star, A_csr, K)
        bridges, ev = extract_fiedler_bridges_sparse(L_sync, A_csr,
                                                     k_eigs=k_eigs, top_k=top_k)
        bridges_by_K[K_mult] = bridges
        theta_by_K[K_mult]   = theta_star
        eigvals_by_K[K_mult] = ev
        print(f'  K={K_mult:.1f}*K_stable={K:.3f}: {len(bridges)} bridges, '
              f'lambda_2={ev[1]:.5f}, r={global_order_parameter(theta_star):.3f}, '
              f't={time.time()-t0:.1f}s')
    return bridges_by_K, theta_by_K, eigvals_by_K


def jaccard(a, b):
    sa = {(int(e[0]), int(e[1])) for e in a}
    sb = {(int(e[0]), int(e[1])) for e in b}
    return len(sa & sb) / max(len(sa | sb), 1)


print('Pipeline helpers defined (v04: K_stable budget enforced).')""")

code("""# K_stable on the TF-IDF adjacency at the headline tau (guarded by precheck).
rng_canon    = np.random.default_rng(GLOBAL_SEED)
omega_corp   = rng_canon.normal(0.0, 1.0, size=N); omega_corp -= omega_corp.mean()
theta0_fixed = np.random.default_rng(GLOBAL_SEED).uniform(0, 2*np.pi, size=N)

K_stable, bisect_trace, KSTABLE_STATUS, KSTABLE_WALL_S = None, [], 'NOT RUN (precheck failed)', None
if RUN_DYNAMICS:
    A = A_by_tau[TAU_HEAD]
    print(f'Computing K_stable via bisection on TF-IDF adjacency (tau={TAU_HEAD}, budget {KSTABLE_BUDGET_S:.0f}s)...')
    t_start = time.time()
    try:
        K_stable, bisect_trace = compute_K_stable(A, omega_corp, theta0_fixed)
        KSTABLE_WALL_S = time.time() - t_start
        KSTABLE_STATUS = f'completed in {KSTABLE_WALL_S:.0f}s'
        print(f'K_stable = {K_stable:.4f}  ({len(bisect_trace)} probes, {KSTABLE_WALL_S:.0f}s)')
    except KStableBudgetExceeded as e:
        KSTABLE_WALL_S = time.time() - t_start
        KSTABLE_STATUS = f'BUDGET EXCEEDED ({KSTABLE_WALL_S:.0f}s)'
        print(f'[FAIL] {e}')
    except ValueError as e:
        KSTABLE_WALL_S = time.time() - t_start
        KSTABLE_STATUS = f'BRACKET DEGENERATE: {e}'
        print(f'[FAIL] {e}')
else:
    print('[SKIP] Connectivity precheck failed at all tau — K_stable not attempted.')

DYNAMICS_OK = RUN_DYNAMICS and (K_stable is not None)""")

code("""# Canonical K-sweep (guarded).
if DYNAMICS_OK:
    print(f'Running canonical K-sweep ({K_MULTS_V03}) on TF-IDF corpus graph (N={N}, tau={TAU_HEAD})...')
    t0_sweep = time.time()
    bridges_corp, theta_corp, eigvals_corp = run_pipeline_sparse(
        A, omega_corp, K_stable, K_MULTS_V03, theta0_fixed, n_communities, top_k=200
    )
    print(f'Sweep done in {time.time()-t0_sweep:.0f}s.')
    print()
    print(f'Top-5 Fiedler bridges at K={MID_K}*K_stable:')
    for rank, (i, j, c) in enumerate(bridges_corp[MID_K][:5], 1):
        ci, cj = int(community_ids[i]), int(community_ids[j])
        print(f'  #{rank}: node {nodes[i]["id"]} (comm {ci}) -- {nodes[j]["id"]} (comm {cj})  contrib={c:.5f}')
else:
    bridges_corp, theta_corp, eigvals_corp = {}, {}, {}
    print('[SKIP] dynamics not run.')""")

code("""# SC3: r_global(K=2.0*K_stable) > 0.9  PASS | 0.7-0.9 FLAG | <0.7 ABORT
if DYNAMICS_OK:
    r_2K = global_order_parameter(theta_corp[2.0])
    print(f'SC3: r_global at K=2.0*K_stable ({2.0*K_stable:.3f}) = {r_2K:.4f}')
    if r_2K > 0.9:    SC3 = 'PASS'
    elif r_2K > 0.7:  SC3 = f'FLAG (r={r_2K:.4f}; partial lock — expected for sparse real-world graphs)'
    else:              SC3 = f'ABORT (r={r_2K:.4f} < 0.7 — ODE not converging; check integration params)'
    print(f'SC3 verdict: {SC3}')
    assert 'ABORT' not in SC3, SC3
else:
    SC3, r_2K = 'SKIP (precheck failed)', None
    print('[SKIP] SC3.')""")

code("""# SC5: lambda_2 >= 0 in upper cluster; spectral gap lambda_3/lambda_2 >= 1.05
if DYNAMICS_OK:
    upper_cluster = [K for K in K_MULTS_V03 if eigvals_corp[K][1] >= 0]
    lower_cluster = [K for K in K_MULTS_V03 if eigvals_corp[K][1] < 0]
    print(f'SC5: upper-cluster K_mults (lambda_2 >= 0): {upper_cluster}')
    print(f'     lower-cluster K_mults (lambda_2 < 0):  {lower_cluster}')

    lam2_violations = [K for K in upper_cluster if eigvals_corp[K][1] < -1e-6]
    ev_mid    = eigvals_corp[MID_K]
    lambda_2  = float(ev_mid[1])
    lambda_3  = float(ev_mid[2]) if len(ev_mid) > 2 else float('nan')
    gap       = lambda_3 / lambda_2 if lambda_2 > 1e-9 else float('nan')
    print(f'  At K={MID_K}: lambda_2={lambda_2:.6f}, lambda_3={lambda_3:.6f}, gap={gap:.4f}')

    if lam2_violations:
        SC5 = f'ABORT (lambda_2 < 0 at upper-cluster K={lam2_violations})'
    elif gap != gap or gap < 1.05:
        SC5 = f'FLAG (lambda_2={lambda_2:.4f}>0 OK, gap={gap:.4f} < 1.05 — Fiedler direction ambiguous)'
    else:
        SC5 = f'PASS (lambda_2={lambda_2:.4f} > 0, gap={gap:.4f} >= 1.05)'
    print(f'SC5 verdict: {SC5}')
    assert 'ABORT' not in SC5, SC5
else:
    SC5, upper_cluster = 'SKIP (precheck failed)', []
    print('[SKIP] SC5.')""")

code("""# SC2: permuted-omega, fixed theta0 — bridges should be topology-driven.
if DYNAMICS_OK:
    omega_perm = np.random.default_rng(GLOBAL_SEED + 1).permutation(omega_corp)
    print('Running permuted-omega sweep (theta0 fixed)...')
    t0_perm = time.time()
    bridges_perm, _, _ = run_pipeline_sparse(
        A, omega_perm, K_stable, K_MULTS_V03, theta0_fixed, n_communities, top_k=200
    )
    print(f'Done in {time.time()-t0_perm:.0f}s.')

    jac_by_K = {K: jaccard(bridges_corp[K], bridges_perm[K]) for K in upper_cluster}
    jac_vals  = list(jac_by_K.values()) if jac_by_K else [0.0]
    jac_min   = min(jac_vals)
    jac_mean  = float(np.mean(jac_vals))
    print('Plateau Jaccard (canonical vs permuted-omega), upper cluster:')
    for K, j in jac_by_K.items():
        print(f'  K_mult={K}: Jaccard={j:.4f}')
    print(f'  mean={jac_mean:.4f}, min={jac_min:.4f}')

    if jac_min >= 0.85:   SC2 = 'PASS'
    elif jac_min >= 0.50: SC2 = f'FLAG (min_J={jac_min:.4f} — moderate omega sensitivity)'
    else:                 SC2 = f'FLAG (min_J={jac_min:.4f} — high omega sensitivity; failure mode 4)'
    print(f'SC2 verdict: {SC2}')
else:
    SC2 = 'SKIP (precheck failed)'
    print('[SKIP] SC2.')""")

code("""# SC1a: degree-matched Erdos-Renyi null.
if DYNAMICS_OK:
    def build_er_null(A_csr, seed=99):
        rng_er = np.random.default_rng(seed)
        n      = A_csr.shape[0]
        ne     = A_csr.nnz // 2
        p      = 2 * ne / max(n * (n - 1), 1)
        rows, cols = [], []
        for i in range(n):
            for j in range(i + 1, n):
                if rng_er.random() < p:
                    rows += [i, j]; cols += [j, i]
        data = np.ones(len(rows))
        return csr_matrix((data, (rows, cols)), shape=(n, n))

    print('Building ER null graph...')
    A_er = build_er_null(A, seed=99)
    print(f'ER null: N={A_er.shape[0]}, edges={A_er.nnz//2}')

    print('Running ER null K-sweep...')
    t0_er = time.time()
    bridges_er, _, _ = run_pipeline_sparse(
        A_er, omega_corp, K_stable, K_MULTS_V03, theta0_fixed, n_communities, top_k=200
    )
    print(f'Done in {time.time()-t0_er:.0f}s.')
    jac_er = jaccard(bridges_corp[MID_K], bridges_er[MID_K])
    print(f'SC1a Jaccard(corpus, ER-null) at K={MID_K}: {jac_er:.4f}')
    if jac_er <= 0.30:    SC1A = f'PASS (J={jac_er:.4f} <= 0.3 — topology-sensitive)'
    elif jac_er <= 0.60:  SC1A = f'FLAG (J={jac_er:.4f} in (0.3, 0.6])'
    else:                 SC1A = f'ABORT (J={jac_er:.4f} > 0.6 — bridges are topology-artefacts on ER null)'
    print(f'SC1a verdict: {SC1A}')
    assert 'ABORT' not in SC1A, SC1A
else:
    SC1A, bridges_er = 'SKIP (precheck failed)', {}
    print('[SKIP] SC1a.')""")

code("""# SC1b: degree-preserving configuration-model null.
if DYNAMICS_OK:
    def build_config_model_null(A_csr, seed=77):
        \"\"\"Chung-Lu configuration model: random pairing of degree-matched stubs.\"\"\"
        rng_cm = np.random.default_rng(seed)
        n      = A_csr.shape[0]
        deg    = np.array((A_csr != 0).sum(axis=1)).flatten().astype(int)
        stubs  = np.repeat(np.arange(n), deg)
        rng_cm.shuffle(stubs)
        if len(stubs) % 2 != 0:
            stubs = stubs[:-1]
        rows, cols = [], []
        seen = {}
        for k in range(0, len(stubs) - 1, 2):
            i, j = int(stubs[k]), int(stubs[k+1])
            if i != j:
                edge = (min(i, j), max(i, j))
                if edge not in seen:
                    rows += [i, j]; cols += [j, i]
                    seen[edge] = True
        data = np.ones(len(rows))
        return csr_matrix((data, (rows, cols)), shape=(n, n))

    print('Building configuration-model null graph...')
    A_cm = build_config_model_null(A, seed=77)
    print(f'Config-model null: N={A_cm.shape[0]}, edges={A_cm.nnz//2} (real: {A.nnz//2})')

    print('Running config-model null K-sweep...')
    t0_cm = time.time()
    bridges_cm, _, _ = run_pipeline_sparse(
        A_cm, omega_corp, K_stable, K_MULTS_V03, theta0_fixed, n_communities, top_k=200
    )
    print(f'Done in {time.time()-t0_cm:.0f}s.')
    jac_cm = jaccard(bridges_corp[MID_K], bridges_cm[MID_K])
    print(f'SC1b Jaccard(corpus, config-model-null) at K={MID_K}: {jac_cm:.4f}')
    if jac_cm <= 0.30:    SC1B = f'PASS (J={jac_cm:.4f} <= 0.3 — beyond-degree topology detected)'
    elif jac_cm <= 0.60:  SC1B = f'FLAG (J={jac_cm:.4f} — moderate degree-distribution overlap)'
    else:                 SC1B = f'ABORT (J={jac_cm:.4f} > 0.6 — bridges fully explained by degree sequence)'
    print(f'SC1b verdict: {SC1B}')
    assert 'ABORT' not in SC1B, SC1B
else:
    SC1B, bridges_cm = 'SKIP (precheck failed)', {}
    print('[SKIP] SC1b.')""")

code("""# Plateau criterion: Jaccard > 0.7 within upper cluster.
if DYNAMICS_OK:
    plateau = [
        K for K in upper_cluster
        if all(jaccard(bridges_corp[K], bridges_corp[K2]) > 0.70
               for K2 in upper_cluster if K2 != K)
    ]
    if not plateau:
        plateau     = upper_cluster   # fallback
        PLATEAU_V   = 'FLAG (no strict plateau; using full upper cluster)'
    else:
        PLATEAU_V   = f'PASS ({len(plateau)}/{len(upper_cluster)} upper-cluster K form stable plateau)'

    print(f'Plateau verdict: {PLATEAU_V}')
    print(f'Plateau K_mults: {plateau}')

    jac_plateau_min = min(
        jaccard(bridges_corp[K1], bridges_corp[K2])
        for K1 in upper_cluster for K2 in upper_cluster if K1 != K2
    ) if len(upper_cluster) >= 2 else 1.0
    print(f'Min plateau Jaccard: {jac_plateau_min:.4f} (gate >= {PLATEAU_JAC_MIN})')
else:
    PLATEAU_V, jac_plateau_min = 'SKIP (precheck failed)', None
    print('[SKIP] plateau.')""")

code("""# Build normalised TF-IDF matrix from corpus (unconditional; cheap).
# tfidf_vec is [[term, score], ...] from resyn exporter.
print('Building TF-IDF matrix...')
all_terms = set()
for n in nodes:
    for term, _score in n['tfidf_vec']:
        all_terms.add(term)
vocab       = sorted(all_terms)
term_to_idx = {t: i for i, t in enumerate(vocab)}
V           = len(vocab)
print(f'Vocabulary: {V} terms across {N} nodes')

tfidf_lil = lil_matrix((N, V), dtype=np.float32)
for i, nd in enumerate(nodes):
    for term, score in nd['tfidf_vec']:
        j = term_to_idx.get(term)
        if j is not None:
            tfidf_lil[i, j] = score

tfidf_csr  = tfidf_lil.tocsr()
row_norms  = np.array(np.sqrt(tfidf_csr.multiply(tfidf_csr).sum(axis=1))).flatten()
row_norms  = np.maximum(row_norms, 1e-12)
tfidf_norm = diags(1.0 / row_norms) @ tfidf_csr  # L2-normalised rows
print('TF-IDF matrix ready.')


def tfidf_cosine(i, j):
    vi = np.asarray(tfidf_norm[i].todense()).flatten()
    vj = np.asarray(tfidf_norm[j].todense()).flatten()
    return float(np.dot(vi, vj))""")

code("""# Feynman benchmark. Corpus presence + n_eval are substrate-independent and
# always computed (needed for the kill-gate check); detection/BENCH_P10 requires dynamics.
with open(FEYNMAN_FILE) as f:
    feynman = json.load(f)

corpus_ids = set(node_idx.keys())

def evaluate_pair(pair, bridges_list, k_eval):
    a_id = pair['side_a']['arxiv_id']
    b_id = pair['side_b']['arxiv_id']
    a_in = a_id in corpus_ids
    b_in = b_id in corpus_ids
    result = {
        'pair_id': pair['id'],
        'a_id': a_id, 'b_id': b_id,
        'a_in': a_in, 'b_in': b_in,
        'detected': None, 'cosine': None,
    }
    if not (a_in and b_in):
        return result
    ai       = node_idx[a_id]
    bi_      = node_idx[b_id]
    a_comm   = int(community_ids[ai])
    b_comm   = int(community_ids[bi_])
    cosine   = tfidf_cosine(ai, bi_)
    result['a_comm']  = a_comm
    result['b_comm']  = b_comm
    result['cosine']  = cosine
    detected = False
    if bridges_list:
        for i, j, _c in bridges_list[:k_eval]:
            ci = int(community_ids[i])
            cj = int(community_ids[j])
            if set([ci, cj]) == set([a_comm, b_comm]):
                detected = True; break
        result['detected'] = detected
    return result

bench_bridges = bridges_corp.get(MID_K, []) if DYNAMICS_OK else []
bench_results = [evaluate_pair(p, bench_bridges, PRECISION_AT_K) for p in feynman['pairs']]

print('Feynman benchmark — corpus presence:')
for r in bench_results:
    status = 'BOTH' if (r['a_in'] and r['b_in']) else ('A-only' if r['a_in'] else ('B-only' if r['b_in'] else 'ABSENT'))
    print(f'  {r["pair_id"]}: {status}  ({r["a_id"]} / {r["b_id"]})')

evaluable = [r for r in bench_results if r['a_in'] and r['b_in']]
n_eval    = len(evaluable)
print(f'\\nEvaluable pairs (both sides in corpus): {n_eval}/{len(bench_results)}')

BENCH_P10, BENCH_COSINE, BENCH_V = None, None, None
if not DYNAMICS_OK:
    BENCH_V = 'NOT PRODUCED (connectivity precheck failed — dynamics never ran)'
    if evaluable:
        BENCH_COSINE = float(np.mean([r['cosine'] for r in evaluable]))
        print(f'TF-IDF cosine mean over evaluable pairs (diagnostic): {BENCH_COSINE:.4f}')
    print(f'\\nBenchmark verdict: {BENCH_V}')
elif n_eval == 0:
    print('[WARNING] No pairs evaluable — corpus likely not built from a relevant seed.')
    BENCH_P10 = 0.0; BENCH_COSINE = 0.0
    BENCH_V = 'ABORT (n_eval = 0)'
else:
    n_detected   = sum(r['detected'] for r in evaluable)
    BENCH_P10    = n_detected / PRECISION_AT_K  # denominator is k, not n_eval
    BENCH_COSINE = float(np.mean([r['cosine'] for r in evaluable]))

    print(f'\\nBenchmark results at K={MID_K}*K_stable (tau={TAU_HEAD}):')
    print(f'  precision@{PRECISION_AT_K} = {BENCH_P10:.3f}  (target >= {PRECISION_TARGET})')
    print(f'  TF-IDF cosine mean        = {BENCH_COSINE:.4f}  (expected baseline ~ {TFIDF_BASELINE_EXPECT})')
    print()
    print(f'  {"Pair":<40}  {"Detected":>8}  {"Cosine":>7}  {"Comms":>10}')
    print('  ' + '-'*75)
    for r in evaluable:
        comms = f'{r.get("a_comm","?")}-{r.get("b_comm","?")}'
        det   = 'YES' if r['detected'] else 'NO'
        print(f'  {r["pair_id"]:<40}  {det:>8}  {r["cosine"]:>7.4f}  {comms:>10}')

    if BENCH_P10 >= PRECISION_TARGET:
        BENCH_V = f'PASS (precision@{PRECISION_AT_K}={BENCH_P10:.3f} >= {PRECISION_TARGET})'
    elif BENCH_P10 > TFIDF_BASELINE_EXPECT:
        BENCH_V = f'FLAG (precision={BENCH_P10:.3f} > baseline {TFIDF_BASELINE_EXPECT} but < target {PRECISION_TARGET})'
    else:
        BENCH_V = f'ABORT (precision={BENCH_P10:.3f} <= baseline {TFIDF_BASELINE_EXPECT} — no signal above chance)'
    print(f'\\nBenchmark verdict: {BENCH_V}')
    assert 'ABORT' not in BENCH_V, f'ABORT: {BENCH_V}. Kuramoto-LBD shows no signal on Feynman benchmark.'""")

code("""# Null-graph benchmark (topology-specificity). Guarded.
if DYNAMICS_OK:
    null_results_er = [evaluate_pair(p, bridges_er[MID_K], PRECISION_AT_K)
                       for p in feynman['pairs']]
    null_results_cm = [evaluate_pair(p, bridges_cm[MID_K], PRECISION_AT_K)
                       for p in feynman['pairs']]

    eval_er = [r for r in null_results_er if r['a_in'] and r['b_in']]
    eval_cm = [r for r in null_results_cm if r['a_in'] and r['b_in']]

    p10_er = sum(r['detected'] for r in eval_er) / PRECISION_AT_K if eval_er else 0.0
    p10_cm = sum(r['detected'] for r in eval_cm) / PRECISION_AT_K if eval_cm else 0.0

    print(f'Null benchmark precision@{PRECISION_AT_K}:')
    print(f'  ER null:           {p10_er:.3f}  (abort gate: > {NULL_INFLATE_ABORT})')
    print(f'  Config-model null: {p10_cm:.3f}  (abort gate: > {NULL_INFLATE_ABORT})')
    print(f'  Real corpus:       {BENCH_P10:.3f}')

    NULL_GATE = 'PASS' if (p10_er <= NULL_INFLATE_ABORT and p10_cm <= NULL_INFLATE_ABORT) else 'ABORT'
    print(f'Null gate: {NULL_GATE}')
    assert NULL_GATE != 'ABORT', f'ABORT: null precision too high. Method may be detecting graph artefacts.'
else:
    p10_er, p10_cm, NULL_GATE = None, None, 'SKIP (precheck failed)'
    print('[SKIP] null benchmark.')""")

code("""# Orthogonality check vs ABC bridges (optional input). Guarded.
if not DYNAMICS_OK:
    ORTHO_V = 'SKIP (precheck failed)'
    print('[SKIP] orthogonality check.')
elif not os.path.exists(ABC_BRIDGES):
    print(f'ABC bridges file not found: {ABC_BRIDGES}')
    print('Skipping orthogonality check. Export resyn bridges first to enable.')
    ORTHO_V = 'SKIP'
else:
    with open(ABC_BRIDGES) as f:
        abc_data = json.load(f)
    abc_set = set()
    for e in abc_data[:100]:
        src, dst = e.get('src',''), e.get('dst','')
        if src in node_idx and dst in node_idx:
            i, j = node_idx[src], node_idx[dst]
            abc_set.add((min(i,j), max(i,j)))
    kur_set = {(min(int(e[0]),int(e[1])), max(int(e[0]),int(e[1])))
               for e in bridges_corp[MID_K][:100]}
    jac_ortho = len(kur_set & abc_set) / max(len(kur_set | abc_set), 1)
    print(f'Orthogonality Jaccard(Kuramoto, ABC): {jac_ortho:.4f}')
    if jac_ortho > ORTHO_RETIRE_JAC:
        ORTHO_V = f'FLAG-RETIRE (J={jac_ortho:.4f} > {ORTHO_RETIRE_JAC})'
    elif jac_ortho > 0.30:
        ORTHO_V = f'FLAG (J={jac_ortho:.4f} — partial overlap; novel signal present)'
    else:
        ORTHO_V = f'PASS (J={jac_ortho:.4f} <= 0.30 — Kuramoto bridges are largely novel)'
    print(f'Orthogonality verdict: {ORTHO_V}')""")

code("""# Summary + verdict against LOCKED EXP-RS-11 predictions.
print('=' * 80)
print('KURAMOTO-LBD v0.4 — EXP-RS-11 RESULTS SUMMARY (TF-IDF semantic-edge substrate)')
print('=' * 80)
print(f'Corpus: N={N}, {n_communities} communities')
print(f'Substrate: TF-IDF cosine edges (edge iff cosine >= tau, weight = cosine)')
print()
print('Connectivity sweep (precheck n_cc/N <= 0.05):')
print(f'  {"tau":>5} {"n_edges":>8} {"n_cc":>5} {"largest_cc":>10} {"ncc/N":>7}  precheck')
for row in sweep_recomputed:
    ok = row['ncc_frac'] <= PRECHECK_MAX_NCC_FRAC
    print(f'  {row["tau"]:>5.2f} {row["n_edges"]:>8d} {row["n_cc"]:>5d} {row["largest_cc_size"]:>10d} '
          f'{row["ncc_frac"]:>7.3f}  {"PASS" if ok else "FAIL"}')
print()

row_03 = next(r for r in sweep_recomputed if abs(r['tau'] - 0.3) < 1e-9)
p1_met = row_03['ncc_frac'] <= 0.05
p2_met = row_03['largest_cc_size'] >= 0.80 * N
p3_met = (KSTABLE_STATUS.startswith('completed')
          and KSTABLE_WALL_S is not None and KSTABLE_WALL_S <= KSTABLE_BUDGET_S)
gate_met = (n_eval >= 3) and (BENCH_P10 is not None and BENCH_P10 > 0.15)

print('LOCKED PREDICTIONS (EXP-RS-11, pre-registered — evaluated, not adjusted):')
print(f'  P-1  tau=0.3 n_cc/N <= 0.05:      actual {row_03["ncc_frac"]:.3f} '
      f'({row_03["n_cc"]} cc / {N})            {"MET" if p1_met else "NOT MET"}')
print(f'  P-2  tau=0.3 largest CC >= 80%:   actual {row_03["largest_cc_size"]/N:.1%} '
      f'({row_03["largest_cc_size"]}/{N})              {"MET" if p2_met else "NOT MET"}')
print(f'  P-3  K_stable completes <= 300s:  {KSTABLE_STATUS:<32} {"MET" if p3_met else "NOT MET"}')
print(f'  P-4  gate n_eval>=3 & P10>0.15:   n_eval={n_eval}, '
      f'BENCH_P10={BENCH_P10 if BENCH_P10 is not None else "not produced"}   {"MET" if gate_met else "NOT MET"}')
print()

if DYNAMICS_OK:
    print('Sanity checks:')
    for name, v in [('SC4-TOY', SC4_TOY), ('SC3', SC3), ('SC5', SC5), ('SC2', SC2),
                    ('SC1a', SC1A), ('SC1b', SC1B), ('PLATEAU', PLATEAU_V),
                    ('NULL', NULL_GATE), ('ORTHO', ORTHO_V)]:
        print(f'  {name:<8} {v}')
    print()

print('=' * 80)
if not RUN_DYNAMICS:
    print('OVERALL: PRECHECK FAIL at all tau in {0.2, 0.3, 0.4, 0.5}.')
    print('BENCH_P10 was NOT produced; benchmark gate NOT met.')
    print('=> EXP-RS-11 pivot kill gate FIRES (semantic-edge substrate cannot support')
    print('   dynamical LBD at pre-registered thresholds on this corpus).')
    print('   Per pre-registered follow-ups: corpus itself too narrow; Path B (seed')
    print('   selection) is the remaining option. Kill decision -> human via vault.')
elif not gate_met:
    print(f'OVERALL: FAIL — benchmark gate not met (n_eval={n_eval}, BENCH_P10={BENCH_P10}).')
    print('=> EXP-RS-11 pivot kill gate FIRES. Kill decision -> human via vault.')
elif BENCH_P10 >= 0.30:
    print(f'OVERALL: SUCCESS — BENCH_P10={BENCH_P10:.3f} >= 0.30 (EXP-RS-06 success bar).')
    print('=> H-RS-substrate discriminating experiment UNBLOCKED; tournament-ready.')
else:
    print(f'OVERALL: PASS-MARGINAL — gate met (BENCH_P10={BENCH_P10:.3f} > 0.15) but below 0.30.')
    print('=> Substrate viable; method signal marginal. Tournament decision -> vault.')
print('=' * 80)""")

nb['cells'] = cells
nb['metadata']['kernelspec'] = {
    'display_name': 'Python 3', 'language': 'python', 'name': 'python3'}
nbf.write(nb, 'kuramoto_lbd_v04.ipynb')
print(f'Wrote kuramoto_lbd_v04.ipynb ({len(cells)} cells)')
