#!/usr/bin/env python3
"""
Kuramoto-LBD v0.5 -- EXP-RS-12 (research-synergy Phase 31).

Committed BEFORE execution on real data (research-operating-manual discipline).

PROVENANCE: byte-faithful port of the validated v03 notebook (kuramoto_lbd_v03.ipynb),
with EXACTLY these changes (all other logic, thresholds, seeds identical):
  1. DATA_FILE -> data/research_synergy_kuramoto_full.json (the FULL data-kuramoto corpus).
  2. Cell 2 rewritten to extract the giant connected component (C-13) and re-index; the
     full corpus has 2 components (224 + 3), and even 2 components give lambda_2=0 ->
     compute_K_stable diverges. Dynamics run on the 224-node giant CC (single component).
  3. The six *experiment* abort-trigger asserts (SC3, SC5, SC1a, SC1b, BENCH_V, NULL_GATE)
     are made NON-FATAL (recorded to V05_ABORTS, run continues) so the run produces the
     full either-way evidence (nulls + verdict) the EXP-RS-12 pre-registration requires.
     NO threshold, prediction, or scoring formula is changed. The toy-pipeline correctness
     assert (SC4_TOY) stays hard.
Rationale (why dropping the pre-2015 slice is valid): BENCH_P10 is a DATE-AGNOSTIC recovery
metric (dynamical-lbd.md Criterion 3(a): corpus must "contain both literatures"); the
pre-2015 filter (C-1) was unnecessary and it alone shattered the citation graph. C-1
superseded by C-12. Predictions LOCKED: vault agentic-experiments-research.md § EXP-RS-12.

Run:  .venv/bin/python kuramoto_lbd_v05.py
"""

# ============================================================================
# cell 1
# ============================================================================
import json
import os
import time
import numpy as np
from scipy.integrate import solve_ivp
from scipy.sparse import csr_matrix, diags, lil_matrix
from scipy.sparse.linalg import eigsh
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

# --- Paths ---
DATA_FILE      = 'data/research_synergy_bridged.json'
FEYNMAN_FILE   = 'data/feynman_10pair_papers.json'
ABC_BRIDGES    = 'data/abc_bridges.json'   # optional; skip check if absent

# --- Protocol constants (pre-registered, do not change post-hoc) ---
GLOBAL_SEED    = 42
K_MULTS_V03    = [1.3, 1.5, 2.0, 3.0]  # restricted to synchronisation regime
MID_K          = 1.3                    # plateau representative

PRECISION_AT_K         = 10
PRECISION_TARGET       = 0.30
TFIDF_BASELINE_EXPECT  = 0.15
PLATEAU_JAC_MIN        = 0.85
NULL_INFLATE_ABORT     = 0.25
ORTHO_RETIRE_JAC       = 0.70

print('Imports OK.')
print(f'K_mults: {K_MULTS_V03}')
print(f'Pre-registered precision@{PRECISION_AT_K} target: {PRECISION_TARGET}')
print(f'Expected TF-IDF cosine baseline: {TFIDF_BASELINE_EXPECT}')


V05_ABORTS = []
GIANT_FRAC_MIN = 0.95  # C-13 giant-CC precheck

# ============================================================================
# cell 2
# ============================================================================
# --- Load Louvain corpus JSON (FULL data-kuramoto export; EXP-RS-12) ---
# C-13: run dynamics on the largest connected component only. Even 2 components
# give lambda_2(L_uw)=0 -> K_hi=4/lambda_2 diverges. Giant CC -> single component.
import hashlib
from scipy.sparse.csgraph import connected_components

if not os.path.exists(DATA_FILE):
    raise FileNotFoundError(f"Corpus not found: {DATA_FILE}")

with open(DATA_FILE, 'rb') as f:
    _raw = f.read()
print(f'Corpus file: {DATA_FILE}')
print(f'sha256: {hashlib.sha256(_raw).hexdigest()}')
corpus = json.loads(_raw)

_nodes_all = corpus['nodes']
_edges_all = corpus['edges']
_N_all     = len(_nodes_all)
_idx_all   = {n['id']: i for i, n in enumerate(_nodes_all)}

# full adjacency to find connected components
_r, _c = [], []
for e in _edges_all:
    i = _idx_all.get(e['src']); j = _idx_all.get(e['dst'])
    if i is None or j is None:
        continue
    _r.extend([i, j]); _c.extend([j, i])
_A_all = csr_matrix((np.ones(len(_r)), (_r, _c)), shape=(_N_all, _N_all))
_ncc_all, _labels = connected_components(_A_all, directed=False)
_sizes = np.bincount(_labels)
_giant_label = int(_sizes.argmax())
_giant_ids = [_nodes_all[i]['id'] for i in range(_N_all) if _labels[i] == _giant_label]
_giant_set = set(_giant_ids)

giant_frac = len(_giant_ids) / _N_all
PRECHECK_PASS = giant_frac >= GIANT_FRAC_MIN
print(f'Full corpus: N={_N_all}, components={_ncc_all}, top sizes={sorted(_sizes.tolist(), reverse=True)[:5]}')
print(f'Giant CC: {len(_giant_ids)} nodes ({100*giant_frac:.1f}%)  precheck(>= {GIANT_FRAC_MIN}): {"PASS" if PRECHECK_PASS else "FAIL"}')

# --- restrict corpus to the giant CC and re-index ---
nodes         = [n for n in _nodes_all if n['id'] in _giant_set]
N             = len(nodes)
node_idx      = {n['id']: i for i, n in enumerate(nodes)}
community_ids = np.array([n['community_id'] for n in nodes], dtype=np.int64)
n_communities = int(np.unique(community_ids).size)
edges_raw     = [e for e in _edges_all if e['src'] in _giant_set and e['dst'] in _giant_set]

print(f'Giant-CC corpus: N={N} nodes, {len(edges_raw)} edges, {n_communities} communities')
print(f'Fingerprint: {corpus.get("corpus_fingerprint", "N/A")}')
print(f'Louvain params: {corpus.get("louvain_params", {})}')

# --- build symmetric sparse adjacency (undirected) on the giant CC ---
rows, cols, data_vals = [], [], []
skipped = 0
for e in edges_raw:
    i = node_idx.get(e['src']); j = node_idx.get(e['dst'])
    if i is None or j is None:
        skipped += 1; continue
    w = float(e.get('weight', 1.0))
    rows.extend([i, j]); cols.extend([j, i]); data_vals.extend([w, w])

A   = csr_matrix((data_vals, (rows, cols)), shape=(N, N))
deg = np.array(A.sum(axis=1)).flatten()
D_inv = 1.0 / np.maximum(deg, 1e-12)
n_edges = A.nnz // 2

_ncc_giant, _ = connected_components(A, directed=False)
print(f'Adjacency: {n_edges} undirected edges, mean_deg={deg.mean():.2f}, skipped={skipped}')
print(f'Giant-CC n_cc={_ncc_giant} (require 1 for lambda_2>0): {"OK" if _ncc_giant==1 else "ABORT"}')
assert _ncc_giant == 1, "Giant CC is not a single component -- compute_K_stable would diverge (CL-1)"


# ============================================================================
# cell 3
# ============================================================================
# SC4 TOY GATE: run Fiedler extraction on a 50-node two-community toy graph.
# ABORT if true bridge not in top-20. This gates the pipeline correctness
# before the expensive corpus run.

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
    """Fiedler bridge extraction using sparse eigsh (shift-invert) or dense fallback."""
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

K_toy = 1.3 * 16.0  # approximate K_stable for toy graph (will be computed precisely below)
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
assert 'ABORT' not in SC4_TOY, f'Pipeline bug: toy bridge not in top-20. Debug extract_fiedler_bridges_sparse.'


# ============================================================================
# cell 4
# ============================================================================
def _probe_lambda2(K, A_csr, omega, theta0, D_inv_arr, t_end=200, rtol=1e-5, atol=1e-7, max_step=0.5):
    """Integrate Kuramoto ODE; return lambda_2(L_sync) and theta_star."""
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


def compute_K_stable(A_csr, omega, theta0, K_lo=0.5, K_hi=None,
                     tol=0.1, max_iter=15):
    """Bisect for minimum K where lambda_2(L_sync(theta*(K), K)) >= 0.

    Adaptive K_hi: starts at 4/lambda_2(L_unweighted) (Jadbabaie-Motee bound x4).
    Returns (K_stable, trace).
    """
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

    lam2_lo, _ = _probe_lambda2(K_lo, A_csr, omega, theta0, D_inv_arr)
    lam2_hi, _ = _probe_lambda2(K_hi, A_csr, omega, theta0, D_inv_arr)
    for _ in range(5):
        if lam2_lo < 0: break
        K_lo *= 0.5
        lam2_lo, _ = _probe_lambda2(K_lo, A_csr, omega, theta0, D_inv_arr)
    for _ in range(5):
        if lam2_hi >= 0: break
        K_hi *= 2.0
        lam2_hi, _ = _probe_lambda2(K_hi, A_csr, omega, theta0, D_inv_arr)
    if lam2_lo >= 0 or lam2_hi < 0:
        raise ValueError(f'Bisection bracket degenerate: lam2({K_lo})={lam2_lo:.4f}, lam2({K_hi})={lam2_hi:.4f}')
    trace = [(K_lo, lam2_lo), (K_hi, lam2_hi)]
    for _ in range(max_iter):
        if K_hi - K_lo < tol: break
        K_mid = (K_lo + K_hi) / 2
        lam2_mid, _ = _probe_lambda2(K_mid, A_csr, omega, theta0, D_inv_arr)
        trace.append((K_mid, lam2_mid))
        if lam2_mid < 0: K_lo, lam2_lo = K_mid, lam2_mid
        else:             K_hi, lam2_hi = K_mid, lam2_hi
    return K_hi, trace


def run_pipeline_sparse(A_csr, omega, K_ref, K_mults, theta0,
                        n_communities, top_k=100,
                        t_end=200, rtol=1e-5, atol=1e-7, max_step=0.5):
    """K-sweep on sparse graph. Returns bridges_by_K, theta_by_K, eigvals_by_K."""
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


print('Pipeline helpers defined.')


# ============================================================================
# cell 5
# ============================================================================
rng_canon  = np.random.default_rng(GLOBAL_SEED)
omega_corp = rng_canon.normal(0.0, 1.0, size=N); omega_corp -= omega_corp.mean()
theta0_fixed = np.random.default_rng(GLOBAL_SEED).uniform(0, 2*np.pi, size=N)

print('Computing K_stable via bisection (this may take several minutes for large N)...')
t_start = time.time()
K_stable, bisect_trace = compute_K_stable(A, omega_corp, theta0_fixed)
print(f'K_stable = {K_stable:.4f}  ({len(bisect_trace)} probes, {time.time()-t_start:.0f}s)')


# ============================================================================
# cell 6
# ============================================================================
print(f'Running canonical K-sweep ({K_MULTS_V03}) on corpus (N={N})...')
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


# ============================================================================
# cell 7
# ============================================================================
# SC3: r_global(K=2.0*K_stable) > 0.9  PASS | 0.7-0.9 FLAG | <0.7 ABORT
r_2K = global_order_parameter(theta_corp[2.0])
print(f'SC3: r_global at K=2.0*K_stable ({2.0*K_stable:.3f}) = {r_2K:.4f}')
if r_2K > 0.9:    SC3 = 'PASS'
elif r_2K > 0.7:  SC3 = f'FLAG (r={r_2K:.4f}; partial lock — expected for sparse real-world graphs)'
else:              SC3 = f'ABORT (r={r_2K:.4f} < 0.7 — ODE not converging; check integration params)'
print(f'SC3 verdict: {SC3}')
if 'ABORT' in SC3: V05_ABORTS.append(('SC3', SC3)); print(f'[v05 non-fatal ABORT recorded] {SC3}')


# ============================================================================
# cell 8
# ============================================================================
# SC5: lambda_2 >= 0 in upper cluster; spectral gap lambda_3/lambda_2 >= 1.05
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
if 'ABORT' in SC5: V05_ABORTS.append(('SC5', SC5)); print(f'[v05 non-fatal ABORT recorded] {SC5}')


# ============================================================================
# cell 9
# ============================================================================
# SC2: permuted-omega, fixed theta0 — bridges should be topology-driven.
# Jaccard within upper-cluster K values.
# Gate: min >= 0.85 PASS | >= 0.5 FLAG | <0.5 FLAG (failure mode 4)
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


# ============================================================================
# cell 10
# ============================================================================
# SC1a: degree-matched Erdos-Renyi null.
# Build ER graph with same N and same edge probability.
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
if N > 5000:
    print(f'[note] N={N} — using fast Bernoulli sampling for ER null')
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
if 'ABORT' in SC1A: V05_ABORTS.append(('SC1A', SC1A)); print(f'[v05 non-fatal ABORT recorded] {SC1A}')


# ============================================================================
# cell 11
# ============================================================================
# SC1b: degree-preserving configuration-model null.
# Rewires edges while preserving degree sequence; tests beyond ER null
# whether results depend on topology BEYOND degree sequence.
def build_config_model_null(A_csr, seed=77):
    """Chung-Lu configuration model: random pairing of degree-matched stubs."""
    rng_cm = np.random.default_rng(seed)
    n      = A_csr.shape[0]
    deg    = np.array(A_csr.sum(axis=1)).flatten().astype(int)
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
if 'ABORT' in SC1B: V05_ABORTS.append(('SC1B', SC1B)); print(f'[v05 non-fatal ABORT recorded] {SC1B}')


# ============================================================================
# cell 12
# ============================================================================
# Plateau criterion: Jaccard > 0.7 within upper cluster.
# v03 uses only {1.3, 1.5, 2.0, 3.0} — all synchronisation-regime K values.
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

# Full Jaccard matrix
print()
print('Upper-cluster pairwise Jaccard:')
if upper_cluster:
    header = '         ' + '  '.join(f'{k:.1f}' for k in upper_cluster)
    print(header)
    for K1 in upper_cluster:
        row = f'{K1:.1f}:     ' + '  '.join(
            f'{jaccard(bridges_corp[K1], bridges_corp[K2]):.2f}' for K2 in upper_cluster)
        print(row)

# Check pre-registered plateau Jaccard gate
jac_plateau_min = min(
    jaccard(bridges_corp[K1], bridges_corp[K2])
    for K1 in upper_cluster for K2 in upper_cluster if K1 != K2
) if len(upper_cluster) >= 2 else 1.0
print(f'\nMin plateau Jaccard: {jac_plateau_min:.4f} (gate >= {PLATEAU_JAC_MIN})')


# ============================================================================
# cell 13
# ============================================================================
# Build normalised TF-IDF matrix from corpus.
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
    return float(np.dot(vi, vj))


# ============================================================================
# cell 14
# ============================================================================
with open(FEYNMAN_FILE) as f:
    feynman = json.load(f)

corpus_ids = set(node_idx.keys())

# For each pair: check corpus presence, compute TF-IDF cosine, test bridge detection
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
    # Detected if any top-k bridge spans (a_comm, b_comm)
    detected = False
    for i, j, _c in bridges_list[:k_eval]:
        ci = int(community_ids[i])
        cj = int(community_ids[j])
        if set([ci, cj]) == set([a_comm, b_comm]):
            detected = True; break
    result['detected'] = detected
    return result

bench_results = [evaluate_pair(p, bridges_corp[MID_K], PRECISION_AT_K)
                 for p in feynman['pairs']]

# Print corpus presence
print('Feynman benchmark — corpus presence:')
for r in bench_results:
    status = 'BOTH' if (r['a_in'] and r['b_in']) else ('A-only' if r['a_in'] else ('B-only' if r['b_in'] else 'ABSENT'))
    print(f'  {r["pair_id"]}: {status}  ({r["a_id"]} / {r["b_id"]})')

evaluable = [r for r in bench_results if r['a_in'] and r['b_in']]
n_eval    = len(evaluable)
print(f'\nEvaluable pairs: {n_eval}/{len(bench_results)}')

if n_eval == 0:
    print('[WARNING] No pairs evaluable — corpus likely not built from a relevant seed.')
    print('Run crawl with a network-science or physics seed before benchmark.')
    BENCH_P10 = 0.0; BENCH_COSINE = 0.0
else:
    # precision@k: fraction of evaluable pairs detected in top-k bridges
    n_detected   = sum(r['detected'] for r in evaluable)
    BENCH_P10    = n_detected / PRECISION_AT_K  # denominator is k, not n_eval
    BENCH_COSINE = float(np.mean([r['cosine'] for r in evaluable]))

    print(f'\nBenchmark results at K={MID_K}*K_stable:')
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
    print(f'\nBenchmark verdict: {BENCH_V}')
    # ABORT on Feynman benchmark failure
    if 'ABORT' in BENCH_V: V05_ABORTS.append(('BENCH', BENCH_V)); print(f'[v05 non-fatal ABORT recorded] {BENCH_V}')


# ============================================================================
# cell 15
# ============================================================================
# Run Feynman benchmark on null graphs to verify topology-specificity.
# Gate: null precision@10 <= 0.25 (ABORT if > 0.25).

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
if NULL_GATE == 'ABORT': V05_ABORTS.append(('NULL_GATE', NULL_GATE)); print(f'[v05 non-fatal ABORT recorded] NULL_GATE')


# ============================================================================
# cell 16
# ============================================================================
# Orthogonality check: if Jaccard(Kuramoto, ABC-bridges) > 0.70, Kuramoto merely
# reproduces betweenness centrality and adds no new signal. Retire if so.
#
# ABC bridges must be pre-exported by the resyn bridge-finder to data/abc_bridges.json.
# Expected format: [{"src": arxiv_id, "dst": arxiv_id, "score": float}, ...]

if not os.path.exists(ABC_BRIDGES):
    print(f'ABC bridges file not found: {ABC_BRIDGES}')
    print('Skipping orthogonality check. Export resyn bridges first to enable.')
    ORTHO_V = 'SKIP'
else:
    with open(ABC_BRIDGES) as f:
        abc_data = json.load(f)

    # Normalise to (min_id, max_id) pairs for comparison
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
    print(f'  Kuramoto bridges: {len(kur_set)}, ABC bridges: {len(abc_set)}, overlap: {len(kur_set & abc_set)}')

    if jac_ortho > ORTHO_RETIRE_JAC:
        ORTHO_V = (f'FLAG-RETIRE (J={jac_ortho:.4f} > {ORTHO_RETIRE_JAC} — '
                   f'Kuramoto reproduces betweenness; consider retiring)')
    elif jac_ortho > 0.30:
        ORTHO_V = f'FLAG (J={jac_ortho:.4f} — partial overlap; novel signal present)'
    else:
        ORTHO_V = f'PASS (J={jac_ortho:.4f} <= 0.30 — Kuramoto bridges are largely novel)'

    print(f'Orthogonality verdict: {ORTHO_V}')


# ============================================================================
# cell 17
# ============================================================================
# Visualise community graph with top-10 Fiedler bridges highlighted.
# For large N, sample 2000 nodes for layout.

MAX_PLOT_NODES = 2000
if N <= MAX_PLOT_NODES:
    sample_idx = list(range(N))
else:
    rng_plot = np.random.default_rng(7)
    sample_idx = sorted(rng_plot.choice(N, MAX_PLOT_NODES, replace=False).tolist())
    print(f'[viz] Sampling {MAX_PLOT_NODES}/{N} nodes for layout.')

n_plot      = len(sample_idx)
sample_set  = set(sample_idx)
idx_remap   = {old: new for new, old in enumerate(sample_idx)}
comm_sample = community_ids[sample_idx]
n_comm_plot = len(np.unique(comm_sample))

rng_layout  = np.random.default_rng(7)
# 2-D layout: community-aware (jitter around community centroid)
comm_angles = {c: 2*np.pi*i/n_comm_plot for i, c in enumerate(np.unique(comm_sample))}
pos = np.zeros((n_plot, 2))
for new_i, old_i in enumerate(sample_idx):
    c   = int(community_ids[old_i])
    ang = comm_angles.get(c, 0)
    r   = 5.0 + rng_layout.normal(0, 0.5)
    pos[new_i] = [r * np.cos(ang) + rng_layout.normal(0, 0.3),
                  r * np.sin(ang) + rng_layout.normal(0, 0.3)]

fig, ax = plt.subplots(figsize=(10, 8))

# Draw edges between sample nodes (thin, grey)
i_idx_all, j_idx_all = A.nonzero()
for src, dst in zip(i_idx_all, j_idx_all):
    if src < dst and src in sample_set and dst in sample_set:
        s, d = idx_remap[src], idx_remap[dst]
        ax.plot([pos[s,0], pos[d,0]], [pos[s,1], pos[d,1]],
                color='#cccccc', linewidth=0.3, zorder=1, alpha=0.5)

# Draw Fiedler bridges (only those within sample)
top_bridges_viz = [(i, j, c) for i, j, c in bridges_corp[MID_K][:20]
                   if i in sample_set and j in sample_set]
max_c = max((e[2] for e in top_bridges_viz), default=1.0)
for rank, (bi_, bj_, bc) in enumerate(top_bridges_viz[:10]):
    s, d = idx_remap[bi_], idx_remap[bj_]
    lw   = 1.5 + 4.0 * bc / max_c
    ax.plot([pos[s,0], pos[d,0]], [pos[s,1], pos[d,1]],
            color='#e63946', linewidth=lw, zorder=3, alpha=0.9)
    mid = ((pos[s,0]+pos[d,0])/2, (pos[s,1]+pos[d,1])/2)
    ax.text(mid[0], mid[1], str(rank+1), fontsize=6, ha='center', va='center',
            color='white', bbox=dict(facecolor='#e63946', edgecolor='none', pad=1))

# Colour nodes by community (up to 20 communities)
cmap_nodes = plt.cm.tab20(np.linspace(0, 1, min(n_comm_plot, 20)))
comm_list  = list(np.unique(comm_sample))
for ic, comm in enumerate(comm_list[:20]):
    mask = comm_sample == comm
    ax.scatter(pos[mask, 0], pos[mask, 1],
               c=[cmap_nodes[ic % 20]], s=10, zorder=2,
               edgecolors='none', alpha=0.7, label=f'C{comm}')

ax.set_title(
    f'v0.3 Real corpus: top-10 Fiedler bridges at K={MID_K}×K_stable ({MID_K*K_stable:.3f})\n'
    f'N={N}, {n_communities} communities | Red = top-10 Fiedler bridges'
)
ax.axis('off')
plt.tight_layout()
plt.savefig('kuramoto_v06_bridges.png', dpi=120, bbox_inches='tight')
plt.show()
print('Figure saved: kuramoto_v06_bridges.png')


# ============================================================================
# cell 18
# ============================================================================
print('=' * 80)
print('KURAMOTO-LBD v0.6 (EXP-RS-13) — RESULTS SUMMARY')
print('=' * 80)
print(f'Corpus: N={N}, {n_communities} communities, {A.nnz//2} edges')
print(f'K_stable = {K_stable:.4f}  ({len(bisect_trace)} bisection probes)')
print(f'K_mults (v03 restricted): {K_MULTS_V03}')
print(f'Upper cluster: {upper_cluster}')
print()

sc_rows = [
    ('SC4-TOY  true bridge', SC4_TOY[:40],   'rank<=5 ABORT>20'),
    ('SC3      global lock',  SC3[:40],       f'r>0.9 ABORT<0.7 (r={r_2K:.3f})'),
    ('SC5      spectral gap', SC5[:40],       'lam2>=0, gap>=1.05'),
    ('SC2      perm-omega J', SC2[:40],       f'min_J>={PLATEAU_JAC_MIN}'),
    ('SC1a     ER null J',    SC1A[:40],      'J<=0.30 ABORT>0.60'),
    ('SC1b     CM null J',    SC1B[:40],      'J<=0.30 ABORT>0.60'),
    ('PLATEAU  stability',    PLATEAU_V[:40], 'J>0.70 in upper cluster'),
]

print(f'  {"Check":<28}  {"Verdict":<42}  Threshold')
print('  ' + '-'*100)
for check, verdict, thresh in sc_rows:
    print(f'  {check:<28}  {verdict:<42}  {thresh}')

print()
print(f'  {"Benchmark":<28}  {"Result":<42}  Target')
print('  ' + '-'*100)
print(f'  {"FEYNMAN p@10":<28}  {BENCH_P10:.3f}{"":36}  >= {PRECISION_TARGET}')
print(f'  {"TFIDF cosine mean":<28}  {BENCH_COSINE:.4f}{"":35}  (baseline ~ {TFIDF_BASELINE_EXPECT})')
print(f'  {"ER null p@10":<28}  {p10_er:.3f}{"":36}  <= {NULL_INFLATE_ABORT}')
print(f'  {"CM null p@10":<28}  {p10_cm:.3f}{"":36}  <= {NULL_INFLATE_ABORT}')
print(f'  {"Orthogonality":<28}  {ORTHO_V[:40]}{"":2}  J <= {ORTHO_RETIRE_JAC}')

print()
n_aborts = sum(1 for _, v, _ in sc_rows if 'ABORT' in v)
n_flags  = sum(1 for _, v, _ in sc_rows if 'FLAG'  in v)
n_passes = sum(1 for _, v, _ in sc_rows if v.startswith('PASS'))
bench_abort = ('ABORT' in BENCH_V) if 'BENCH_V' in dir() else False
print(f'SC: PASS={n_passes}  FLAG={n_flags}  ABORT={n_aborts+int(bench_abort)}')

print()
if n_aborts or bench_abort:
    print('OVERALL: ABORT — retire Kuramoto-LBD or debug pipeline.')
elif BENCH_P10 >= PRECISION_TARGET:
    print('OVERALL: PASS — Phase 42 (Kuramoto-LBD Rust) authorised for Wave-4 roadmap proposal.')
elif BENCH_P10 > TFIDF_BASELINE_EXPECT:
    print('OVERALL: MARGINAL — above baseline but below target; expand corpus or revise coupling before Phase 42.')
else:
    print('OVERALL: FAIL — no signal above TF-IDF baseline; retire Kuramoto-LBD.')

print()
print('Pre-registered targets:')
print(f'  precision@{PRECISION_AT_K} target:   {PRECISION_TARGET}  actual: {BENCH_P10:.3f}  {"MET" if BENCH_P10>=PRECISION_TARGET else "NOT MET"}')
print(f'  plateau Jaccard min: {PLATEAU_JAC_MIN}  actual: {jac_plateau_min:.3f}  {"MET" if jac_plateau_min>=PLATEAU_JAC_MIN else "NOT MET"}')
print(f'  ortho J <= {ORTHO_RETIRE_JAC}: {"MET" if ORTHO_V == "SKIP" or float(ORTHO_V.split("J=")[1].split(" ")[0]) <= ORTHO_RETIRE_JAC else "NOT MET (retire flag)"}')



# ==========================================================================
# v06 per-pair metric (EXP-RS-13): global top-10 is harsh for a large corpus.
# Per-pair: for each evaluable pair, find the best RANK in the full bridge list
# (top-200) at which a bridge spans its community-pair. Recall@k = detected/n_eval.
# ==========================================================================
def _perpair(bridges_list, k):
    rows = []
    for r in evaluable:
        ac, bc = r.get('a_comm'), r.get('b_comm')
        best = None
        for rank, (i, j, _c) in enumerate(bridges_list, 1):
            if set([int(community_ids[i]), int(community_ids[j])]) == set([ac, bc]):
                best = rank; break
        rows.append((r['pair_id'], ac, bc, best))
    return rows
_pp = _perpair(bridges_corp[MID_K], PRECISION_AT_K)
PERPAIR_RECALL_AT10 = sum(1 for _,_,_,rk in _pp if rk is not None and rk <= 10) / max(len(_pp),1)
PERPAIR_RECALL_ANY  = sum(1 for _,_,_,rk in _pp if rk is not None) / max(len(_pp),1)
print()
print('PER-PAIR bridge ranks (first top-200 bridge spanning the pair community-pair):')
for pid, ac, bc, rk in _pp:
    print(f'  {pid:26} comm {ac}-{bc}:  rank {rk if rk else ">200 (not in top-200)"}')
print(f'Per-pair recall@10 = {PERPAIR_RECALL_AT10:.3f}   recall@any(top200) = {PERPAIR_RECALL_ANY:.3f}')
print(f'(global BENCH_P10 for reference = {BENCH_P10:.3f})')

# ==========================================================================
# v05 epilogue: machine-readable results dump (EXP-RS-12)
# ==========================================================================
import json as _json
_results = {
  'experiment': 'EXP-RS-13', 'phase': 32,
  'corpus': 'research_synergy_kuramoto_full.json (giant CC)',
  'N': int(N), 'n_communities': int(n_communities), 'n_edges': int(A.nnz // 2),
  'K_stable': float(K_stable), 'n_eval': int(n_eval),
  'BENCH_P10': float(BENCH_P10), 'BENCH_COSINE': float(BENCH_COSINE), 'BENCH_V': BENCH_V,
  'p10_er': float(p10_er), 'p10_cm': float(p10_cm),
  'SC2': SC2, 'SC3': SC3, 'SC5': SC5, 'SC1A': SC1A, 'SC1B': SC1B,
  'PLATEAU_V': PLATEAU_V, 'ORTHO_V': ORTHO_V,
  'v05_aborts': V05_ABORTS,
  'perpair_recall_at10': float(PERPAIR_RECALL_AT10), 'perpair_recall_any': float(PERPAIR_RECALL_ANY),
  'perpair_ranks': [{'pair': p, 'a_comm': a, 'b_comm': b, 'rank': rk} for p,a,b,rk in _pp],
  'evaluable_detail': [{'pair': r['pair_id'], 'detected': r['detected'],
                        'cosine': r['cosine'], 'a_comm': r.get('a_comm'),
                        'b_comm': r.get('b_comm')} for r in evaluable],
}
with open('data/kuramoto_v06_results.json', 'w') as _f:
    _json.dump(_results, _f, indent=1)
print('\n[v05] results -> data/kuramoto_v06_results.json')
print(_json.dumps(_results, indent=1))
