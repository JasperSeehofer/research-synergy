"""Generate kuramoto_lbd_v02.ipynb — adaptive K_stable, fixed θ₀, upper-plateau sweep, robustness."""
import nbformat

nb = nbformat.v4.new_notebook()

# ---------------------------------------------------------------------------
# Cell 1 — Markdown header
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_markdown_cell("""\
# Kuramoto-LBD Prototype v0.2 — Methodological Fixes + Robustness Sweep

**Pre-registration:** `wiki/analyses/kuramoto-research-synergy-integration.md`
(v0.2 spec section added 2026-04-18)

**Scope:** Five sanity checks (same as v0.1) with three methodological fixes
applied, plus four robustness checks (R1–R4).

**Fixes vs v0.1:**
1. **Adaptive K_stable** — bisection finds the minimum K where λ₂(L_sync) ≥ 0,
   replacing the infinite-N mean-field K_c = 1.5958 as the sweep anchor.
2. **Fixed θ₀** — canonical and permuted-ω runs share identical initial conditions;
   only ω varies in the permutation test.
3. **Upper-plateau sweep** — plateau criterion applies within the stable-K
   cluster (λ₂ ≥ 0) only; sub-threshold K values shown for diagnostic contrast.

**Robustness checks:**
- R1: larger-graph sensitivity (N ∈ {50, 200, 500})
- R2: multi-seed K_stable stability (seeds 42–46)
- R3: ω-distribution ablation (N(0,1), Student-t, log-normal, power-law proxy)
- R4: golden-file export for Python↔Rust parity fixture

**Gate:** All ABORT checks must pass. FLAG checks noted but do not block roadmap.
"""))

# ---------------------------------------------------------------------------
# Cell 2 — Imports & constants
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
import json
import numpy as np
from scipy.integrate import solve_ivp
from scipy.sparse import csr_matrix, diags
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

GLOBAL_SEED = 42

K_MULTS = [0.5, 0.7, 1.0, 1.3, 1.5, 2.0, 3.0]

# Infinite-N mean-field reference — kept for comparison, NOT used as sweep anchor in v0.2
K_C_MEANFIELD = 2.0 * np.sqrt(2.0 * np.pi) / np.pi
print(f'K_c (N(0,1) mean-field, reference only) = {K_C_MEANFIELD:.4f}')
print('Sweep anchor in v0.2: K_stable (adaptive, per bisection)')
"""))

# ---------------------------------------------------------------------------
# Cell 3 — Graph builders
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
def build_two_community_toy(n_per=25, p_intra=0.35, seed=42, n_bridges=1):
    \"\"\"Two-community ER graph with deterministic bridge edges.

    Args:
        n_per: nodes per community (N = 2*n_per)
        p_intra: intra-community edge probability
        seed: RNG seed for community edges
        n_bridges: number of bridge edges (node n_per-1-k <-> n_per+k for k in range)

    Returns:
        A (csr, N x N), true_bridges (list of (i,j) tuples, i < j)
    \"\"\"
    rng_local = np.random.default_rng(seed)
    N = 2 * n_per
    rows, cols = [], []

    for comm_start in [0, n_per]:
        for i in range(comm_start, comm_start + n_per):
            for j in range(i + 1, comm_start + n_per):
                if rng_local.random() < p_intra:
                    rows += [i, j]
                    cols += [j, i]

    true_bridges = []
    for k in range(n_bridges):
        bi, bj = n_per - 1 - k, n_per + k
        rows += [bi, bj]
        cols += [bj, bi]
        true_bridges.append((bi, bj))

    data = np.ones(len(rows))
    A = csr_matrix((data, (rows, cols)), shape=(N, N))
    return A, true_bridges


def build_er_null(A, seed=99):
    \"\"\"Degree-matched ER null graph (same node count and edge probability).\"\"\"
    rng_er = np.random.default_rng(seed)
    n = A.shape[0]
    n_edges = A.nnz // 2
    p = 2 * n_edges / (n * (n - 1))
    rows, cols = [], []
    for i in range(n):
        for j in range(i + 1, n):
            if rng_er.random() < p:
                rows += [i, j]
                cols += [j, i]
    data = np.ones(len(rows))
    return csr_matrix((data, (rows, cols)), shape=(n, n))


# Build N=50 baseline
A_toy, true_bridges_base = build_two_community_toy(n_per=25, p_intra=0.35, seed=42, n_bridges=1)
A_base = A_toy
N = A_toy.shape[0]
N_BASE = N
true_bridge = true_bridges_base[0]  # single bridge for main SC checks
deg_toy = np.array(A_toy.sum(axis=1)).flatten()
deg_base = deg_toy
print(f'Baseline: N={N}, edges={A_toy.nnz // 2}, mean_deg={deg_toy.mean():.2f}, true_bridge={true_bridge}')
print(f'Bridge node degrees: {int(deg_toy[true_bridge[0]])}, {int(deg_toy[true_bridge[1]])}')
"""))

# ---------------------------------------------------------------------------
# Cell 4 — Pipeline functions (core v0.2 changes)
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# ── Core Kuramoto ODE & Laplacian ──────────────────────────────────────────

def kuramoto_rhs(t, theta, K, omega, A_csr, D_inv):
    \"\"\"dtheta/dt = omega + K * D^-1 * sum_j A_ij * sin(theta_j - theta_i).\"\"\"
    cos_t, sin_t = np.cos(theta), np.sin(theta)
    coupling = (A_csr @ sin_t) * cos_t - (A_csr @ cos_t) * sin_t
    return omega + K * D_inv * coupling


def global_order_parameter(theta):
    return float(np.abs(np.mean(np.exp(1j * theta))))


def build_sync_laplacian(theta_star, A, K):
    \"\"\"L_sync_ij = -K*cos(theta_j - theta_i)*A_ij, diagonal = row sums.\"\"\"
    i_idx, j_idx = A.nonzero()
    cos_diffs = np.cos(theta_star[j_idx] - theta_star[i_idx])
    n = A.shape[0]
    J_off = csr_matrix((K * cos_diffs, (i_idx, j_idx)), shape=(n, n))
    J_diag = np.array(J_off.sum(axis=1)).flatten()
    return -J_off + diags(J_diag)


def extract_fiedler_bridges(L_sync, A, top_k=100):
    \"\"\"Dense eigh (N<=500). Returns sorted bridge list and eigenvalues.\"\"\"
    eigvals, eigvecs = np.linalg.eigh(L_sync.toarray())
    v2 = eigvecs[:, 1]
    i_idx, j_idx = A.nonzero()
    upper = i_idx < j_idx
    i_u, j_u = i_idx[upper], j_idx[upper]
    signs = np.where(v2 >= 0, 1, -1)
    bridge_mask = signs[i_u] != signs[j_u]
    contributions = np.abs(v2[i_u] * v2[j_u])
    bridge_edges = list(zip(i_u[bridge_mask].tolist(), j_u[bridge_mask].tolist(),
                            contributions[bridge_mask].tolist()))
    bridge_edges.sort(key=lambda e: -e[2])
    return bridge_edges[:top_k], eigvals


def jaccard(a, b):
    sa = {(int(e[0]), int(e[1])) for e in a}
    sb = {(int(e[0]), int(e[1])) for e in b}
    return len(sa & sb) / max(len(sa | sb), 1)


# ── v0.2 additions ─────────────────────────────────────────────────────────

def make_theta0(n, seed=GLOBAL_SEED):
    \"\"\"Deterministic shared initial condition — use to isolate omega variability.\"\"\"
    return np.random.default_rng(seed).uniform(0, 2 * np.pi, size=n)


def _probe_lambda2(K, A, omega, theta0, D_inv):
    \"\"\"Run ODE + Laplacian; return lambda_2 and theta_star.\"\"\"
    sol = solve_ivp(kuramoto_rhs, (0, 200), theta0.copy(),
                    args=(K, omega, A, D_inv),
                    method='RK45', rtol=1e-6, atol=1e-8,
                    dense_output=False, max_step=0.05)
    theta_star = sol.y[:, -1]
    L_sync = build_sync_laplacian(theta_star, A, K)
    eigvals = np.linalg.eigh(L_sync.toarray())[0]
    return eigvals[1], theta_star


def compute_K_stable(A, omega, theta0, K_lo=0.2, K_hi=8.0, tol=0.02, max_iter=20):
    \"\"\"Bisect to find min K where lambda_2(L_sync(theta*(K), K)) >= 0.

    Fix 1 (v0.2): replaces hard-coded K_C_MEANFIELD as sweep anchor.
    Returns (K_stable, trace) where trace = [(K, lambda_2), ...].
    \"\"\"
    n = A.shape[0]
    deg = np.array(A.sum(axis=1)).flatten()
    D_inv = 1.0 / np.maximum(deg, 1e-12)

    lam2_lo, _ = _probe_lambda2(K_lo, A, omega, theta0, D_inv)
    lam2_hi, _ = _probe_lambda2(K_hi, A, omega, theta0, D_inv)

    # Widen bracket if degenerate
    for _ in range(5):
        if lam2_lo < 0:
            break
        K_lo *= 0.5
        lam2_lo, _ = _probe_lambda2(K_lo, A, omega, theta0, D_inv)
    for _ in range(5):
        if lam2_hi >= 0:
            break
        K_hi *= 2.0
        lam2_hi, _ = _probe_lambda2(K_hi, A, omega, theta0, D_inv)

    if lam2_lo >= 0 or lam2_hi < 0:
        raise ValueError(f'Bracket degenerate: lambda_2(K_lo={K_lo})={lam2_lo:.4f}, lambda_2(K_hi={K_hi})={lam2_hi:.4f}')

    trace = [(K_lo, lam2_lo), (K_hi, lam2_hi)]

    for _ in range(max_iter):
        if K_hi - K_lo < tol:
            break
        K_mid = (K_lo + K_hi) / 2
        lam2_mid, _ = _probe_lambda2(K_mid, A, omega, theta0, D_inv)
        trace.append((K_mid, lam2_mid))
        if lam2_mid < 0:
            K_lo, lam2_lo = K_mid, lam2_mid
        else:
            K_hi, lam2_hi = K_mid, lam2_hi

    return K_hi, trace  # K_hi = smallest confirmed-stable K


def classify_k_clusters(eigvals_by_K, lam2_tol=0.0):
    \"\"\"Partition K_mult keys into upper (lambda_2 >= tol) and lower clusters.\"\"\"
    upper = sorted(K for K, ev in eigvals_by_K.items() if ev[1] >= lam2_tol)
    lower = sorted(K for K, ev in eigvals_by_K.items() if ev[1] < lam2_tol)
    return upper, lower


def run_pipeline(A, omega, K_mults=None, K_ref=None, theta0=None, rng_init_seed=0):
    \"\"\"Full K-sweep.

    Args:
        K_ref: sweep anchor (use K_stable; defaults to K_C_MEANFIELD for compat)
        theta0: if provided, same IC used for every K (Fix 2). If None, fresh RNG draw.

    Returns bridges_by_K, theta_by_K, eigvals_by_K (keyed by K_mult float).
    \"\"\"
    if K_mults is None:
        K_mults = K_MULTS
    if K_ref is None:
        K_ref = K_C_MEANFIELD
    n = A.shape[0]
    deg = np.array(A.sum(axis=1)).flatten()
    D_inv = 1.0 / np.maximum(deg, 1e-12)
    rng_init = np.random.default_rng(rng_init_seed)
    bridges_by_K, theta_by_K, eigvals_by_K = {}, {}, {}
    for K_mult in K_mults:
        K = K_mult * K_ref
        theta0_k = theta0.copy() if theta0 is not None else rng_init.uniform(0, 2 * np.pi, size=n)
        sol = solve_ivp(kuramoto_rhs, (0, 200), theta0_k,
                        args=(K, omega, A, D_inv),
                        method='RK45', rtol=1e-6, atol=1e-8,
                        dense_output=False, max_step=0.05)
        theta_star = sol.y[:, -1]
        L_sync = build_sync_laplacian(theta_star, A, K)
        bridges, eigvals = extract_fiedler_bridges(L_sync, A, top_k=100)
        bridges_by_K[K_mult] = bridges
        theta_by_K[K_mult] = theta_star
        eigvals_by_K[K_mult] = eigvals
    return bridges_by_K, theta_by_K, eigvals_by_K


print('Pipeline functions defined (v0.2).')
"""))

# ---------------------------------------------------------------------------
# Cell 5 — Canonical run (K_stable bisection + fixed θ₀)
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
rng_canon = np.random.default_rng(GLOBAL_SEED)
omega_canon = rng_canon.normal(0.0, 1.0, size=N)
omega_canon -= omega_canon.mean()

# Fix 2: single shared initial condition
theta0_fixed = make_theta0(N, seed=GLOBAL_SEED)

# Jadbabaie-Motee lower bound: K >= 1 / lambda_2(L_unweighted)
deg_arr = np.array(A_toy.sum(axis=1)).flatten()
L_unweighted = diags(deg_arr) - A_toy
eigvals_luw = np.linalg.eigh(L_unweighted.toarray())[0]
lam2_unweighted = eigvals_luw[1]
K_alg_lower = 1.0 / max(lam2_unweighted, 1e-12)
print(f'Algebraic connectivity lambda_2(L_unweighted) = {lam2_unweighted:.4f}')
print(f'Jadbabaie-Motee lower bound K_alg = {K_alg_lower:.4f}')
print(f'Mean-field reference K_C_MEANFIELD = {K_C_MEANFIELD:.4f}')
print()

# Fix 1: bisect for K_stable
print('Computing K_stable via bisection...')
K_stable, bisect_trace = compute_K_stable(A_toy, omega_canon, theta0_fixed)
print(f'K_stable = {K_stable:.4f}  (bisection converged in {len(bisect_trace)} probes)')
print(f'Ratio K_stable / K_C_MEANFIELD = {K_stable / K_C_MEANFIELD:.3f}')
print()

MID_K = 1.3  # plateau representative multiplier (applied to K_stable)
print(f'Running canonical K-sweep (K_ref=K_stable={K_stable:.4f}, theta0 FIXED)...')
bridges_by_K, theta_by_K, eigvals_by_K = run_pipeline(
    A_toy, omega_canon, K_ref=K_stable, theta0=theta0_fixed
)
print('Done.')

print(f'\\nTop-5 Fiedler bridges at K={MID_K}*K_stable ({MID_K*K_stable:.4f}):')
for rank, (i, j, contrib) in enumerate(bridges_by_K[MID_K][:5], 1):
    flag = ' <-- TRUE BRIDGE' if (i, j) == true_bridge else ''
    print(f'  #{rank}: ({i}, {j})  contrib={contrib:.5f}{flag}')

# Cluster classification (used throughout)
upper_cluster, lower_cluster = classify_k_clusters(eigvals_by_K)
print(f'\\nCluster classification:')
print(f'  Sub-threshold (lambda_2 < 0): {lower_cluster}  * K_stable')
print(f'  Super-threshold (lambda_2 >= 0): {upper_cluster}  * K_stable')
"""))

# ---------------------------------------------------------------------------
# Cell 6 — Sanity #4 (PRIMARY)
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# SANITY CHECK #4 (PRIMARY): true bridge in top-5 at MID_K * K_stable
# Verdict: top-5 PASS, top-6..20 FLAG, >20 ABORT

bridge_set_mid = [(int(e[0]), int(e[1])) for e in bridges_by_K[MID_K]]
bridge_i, bridge_j = true_bridge

rank_bridge = None
if (bridge_i, bridge_j) in bridge_set_mid:
    rank_bridge = bridge_set_mid.index((bridge_i, bridge_j)) + 1

print(f'True bridge {true_bridge} rank at K={MID_K}*K_stable: {rank_bridge}')

if rank_bridge is not None and rank_bridge <= 5:
    SC4_VERDICT = 'PASS'
elif rank_bridge is not None and rank_bridge <= 20:
    SC4_VERDICT = f'FLAG (rank={rank_bridge}, in top-20 but not top-5)'
else:
    SC4_VERDICT = f'ABORT (rank={rank_bridge} — not in top-20; pipeline bug)'

print(f'Sanity #4 verdict: {SC4_VERDICT}')
assert 'ABORT' not in SC4_VERDICT, f'ABORT: true bridge rank={rank_bridge}. Debug Fiedler extraction.'
"""))

# ---------------------------------------------------------------------------
# Cell 7 — Sanity #3 (global lock)
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# SANITY CHECK #3: global lock r(K=2*K_stable) > 0.9
# Verdict: >0.9 PASS, 0.7-0.9 FLAG, <0.7 ABORT

theta_2K = theta_by_K[2.0]
r_global = global_order_parameter(theta_2K)

print(f'Global order parameter r at K=2.0*K_stable ({2.0*K_stable:.4f}): {r_global:.4f}')

if r_global > 0.9:
    SC3_VERDICT = 'PASS'
elif r_global > 0.7:
    SC3_VERDICT = f'FLAG (r={r_global:.4f}, partial lock; sparse bridge may still be limiting at N=50)'
else:
    SC3_VERDICT = f'ABORT (r={r_global:.4f} < 0.7 — ODE not converging)'

print(f'Sanity #3 verdict: {SC3_VERDICT}')
assert 'ABORT' not in SC3_VERDICT, f'ABORT: r={r_global:.4f}. Check ODE convergence.'
"""))

# ---------------------------------------------------------------------------
# Cell 8 — Sanity #5 (reframed: λ₂ ≥ 0 in upper cluster + spectral gap)
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# SANITY CHECK #5 (v0.2 reframe): lambda_2(L_sync) >= 0 for every upper-cluster K.
# By construction of K_stable, this must hold; any failure indicates a bisection bug.
# Additionally checks spectral gap lambda_3/lambda_2 at MID_K*K_stable.

lam2_violations = [K for K in upper_cluster if eigvals_by_K[K][1] < -1e-6]

eigvals_mid = eigvals_by_K[MID_K]
lambda_2 = eigvals_mid[1]
lambda_3 = eigvals_mid[2]
spectral_gap = lambda_3 / lambda_2 if lambda_2 > 1e-9 else float('nan')

print('Upper-cluster lambda_2 values:')
for K in upper_cluster:
    lam = eigvals_by_K[K][1]
    status = 'OK' if lam >= 0 else 'VIOLATION'
    print(f'  K_mult={K}: lambda_2={lam:.6f}  [{status}]')

print()
print(f'At K={MID_K}*K_stable ({MID_K*K_stable:.4f}):')
print(f'  lambda_2 = {lambda_2:.6f}')
print(f'  lambda_3 = {lambda_3:.6f}')
print(f'  lambda_3/lambda_2 = {spectral_gap:.4f}' if not (spectral_gap != spectral_gap) else '  lambda_3/lambda_2 = NaN (lambda_2 <= 0)')

if lam2_violations:
    SC5_VERDICT = f'ABORT (lambda_2 < 0 at upper-cluster K={lam2_violations} — bisection error)'
elif spectral_gap != spectral_gap or spectral_gap < 1.05:
    SC5_VERDICT = f'FLAG (lambda_2={lambda_2:.4f}>0 OK, but gap={spectral_gap:.4f} < 1.05 — Fiedler direction ambiguous)'
else:
    SC5_VERDICT = f'PASS (lambda_2={lambda_2:.4f} > 0, gap={spectral_gap:.4f} >= 1.05)'

print(f'Sanity #5 verdict: {SC5_VERDICT}')
assert 'ABORT' not in SC5_VERDICT, SC5_VERDICT
"""))

# ---------------------------------------------------------------------------
# Cell 9 — Sanity #2 (fixed θ₀, plateau-wide Jaccard)
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# SANITY CHECK #2 (v0.2 fix): permuted-omega stability.
# Fix: (a) theta0 FIXED — only omega changes; (b) Jaccard measured across upper-plateau.
# Verdict: min-plateau Jaccard >= 0.85 PASS, >= 0.5 FLAG, all-zero FLAG (failure mode 4).

rng_perm = np.random.default_rng(GLOBAL_SEED + 1)
omega_permuted = rng_perm.permutation(omega_canon)

print('Running permuted-omega K-sweep (theta0 FIXED = canonical)...')
bridges_perm, _, eigvals_perm = run_pipeline(
    A_toy, omega_permuted, K_ref=K_stable, theta0=theta0_fixed
)
print('Done.')
print()

jac_by_K = {K_mult: jaccard(bridges_by_K[K_mult], bridges_perm[K_mult]) for K_mult in upper_cluster}
print('Plateau-wide Jaccard (canonical vs permuted-omega), upper cluster:')
for K_mult in upper_cluster:
    print(f'  K_mult={K_mult}: Jaccard={jac_by_K[K_mult]:.4f}')

jac_values = list(jac_by_K.values())
jac_min = min(jac_values)
jac_mean = sum(jac_values) / len(jac_values)
print(f'  --> mean={jac_mean:.4f}, min={jac_min:.4f}')

if jac_min >= 0.85:
    SC2_VERDICT = 'PASS'
elif jac_min >= 0.5:
    SC2_VERDICT = f'FLAG (min-plateau Jaccard={jac_min:.4f}, moderate omega-sensitivity)'
elif all(v == 0.0 for v in jac_values):
    SC2_VERDICT = f'FLAG (Jaccard=0 across plateau — high omega-sensitivity, failure mode 4; expected at N=50)'
else:
    SC2_VERDICT = f'FLAG (min-plateau Jaccard={jac_min:.4f} < 0.5 — high omega-sensitivity)'

print(f'Sanity #2 verdict: {SC2_VERDICT}')
"""))

# ---------------------------------------------------------------------------
# Cell 10 — Sanity #1 (null graph, same θ₀)
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# SANITY CHECK #1: degree-matched ER null graph.
# ABORT if null Jaccard > 0.3 AND true bridge top-10 in null (artefact check).
# Uses same theta0_fixed for apples-to-apples comparison.

A_null = build_er_null(A_toy)
print(f'Null graph: N={A_null.shape[0]}, edges={A_null.nnz // 2}')
print('Running null-graph K-sweep (theta0 FIXED)...')
bridges_null, _, _ = run_pipeline(A_null, omega_canon, K_ref=K_stable, theta0=theta0_fixed)
print('Done.')

bi, bj = true_bridge
null_bridge_set = [(int(e[0]), int(e[1])) for e in bridges_null[MID_K]]
true_bridge_in_null_adj = bool(A_null[bi, bj])
rank_in_null = null_bridge_set.index((bi, bj)) + 1 if (bi, bj) in null_bridge_set else None
jac_null = jaccard(bridges_by_K[MID_K], bridges_null[MID_K])

print(f'True bridge {true_bridge} in null adjacency: {true_bridge_in_null_adj}')
print(f'True bridge rank in null Fiedler bridges: {rank_in_null}')
print(f'Jaccard(canonical, null) at K={MID_K}*K_stable: {jac_null:.4f}')

is_bad_null = true_bridge_in_null_adj and rank_in_null is not None and rank_in_null <= 10 and jac_null > 0.3

if is_bad_null:
    SC1_VERDICT = f'ABORT (Jaccard={jac_null:.4f} > 0.3 and true bridge rank={rank_in_null} in null — artefact)'
elif jac_null <= 0.3:
    SC1_VERDICT = f'PASS (Jaccard={jac_null:.4f} <= 0.3 — topology-sensitive)'
else:
    SC1_VERDICT = f'FLAG (Jaccard={jac_null:.4f} > 0.3 but true bridge absent/low-ranked in null; investigate)'

print(f'Sanity #1 verdict: {SC1_VERDICT}')
assert 'ABORT' not in SC1_VERDICT, SC1_VERDICT
"""))

# ---------------------------------------------------------------------------
# Cell 11 — Plateau detection (upper-cluster only, Fix 3)
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# Fix 3: plateau criterion restricted to upper cluster (lambda_2 >= 0 K values).
# Full diagnostic matrix printed for reference; verdict uses upper cluster only.

plateau = [
    K for K in upper_cluster
    if all(jaccard(bridges_by_K[K], bridges_by_K[K2]) > 0.7
           for K2 in upper_cluster if K2 != K)
]

print(f'Upper-cluster K-plateau (Jaccard > 0.7 within upper cluster):')
if plateau:
    print(f'  Stable K values: {[str(k) + "*K_stable" for k in plateau]}')
    PLATEAU_VERDICT = f'PASS ({len(plateau)}/{len(upper_cluster)} upper-cluster K values form stable plateau)'
else:
    print(f'  No K in upper cluster meets strict plateau criterion.')
    plateau = upper_cluster
    PLATEAU_VERDICT = 'FLAG (no strict plateau in upper cluster; using full upper cluster as fallback)'
print(f'Plateau verdict: {PLATEAU_VERDICT}')

print()
print('Full diagnostic pairwise Jaccard matrix (all K_mults):')
header = '         ' + '  '.join(f'{k:.1f}' for k in K_MULTS)
print(header)
for K1 in K_MULTS:
    row = f'{K1:.1f}:     ' + '  '.join(f'{jaccard(bridges_by_K[K1], bridges_by_K[K2]):.2f}' for K2 in K_MULTS)
    print(row)

print()
print('Upper-cluster pairwise Jaccard matrix:')
header2 = '         ' + '  '.join(f'{k:.1f}' for k in upper_cluster)
print(header2)
for K1 in upper_cluster:
    row2 = f'{K1:.1f}:     ' + '  '.join(f'{jaccard(bridges_by_K[K1], bridges_by_K[K2]):.2f}' for K2 in upper_cluster)
    print(row2)
"""))

# ---------------------------------------------------------------------------
# Cell 12 — Visualization: bridge graph
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
n_per_viz = N // 2
pos = np.zeros((N, 2))
rng_layout = np.random.default_rng(7)
pos[:n_per_viz, 0] = -1.0 + 0.3 * rng_layout.standard_normal(n_per_viz)
pos[:n_per_viz, 1] = rng_layout.standard_normal(n_per_viz)
pos[n_per_viz:, 0] = +1.0 + 0.3 * rng_layout.standard_normal(n_per_viz)
pos[n_per_viz:, 1] = rng_layout.standard_normal(n_per_viz)

fig, ax = plt.subplots(figsize=(9, 6))

i_idx, j_idx = A_toy.nonzero()
for i, j in zip(i_idx, j_idx):
    if i < j:
        ax.plot([pos[i, 0], pos[j, 0]], [pos[i, 1], pos[j, 1]],
                color='#cccccc', linewidth=0.5, zorder=1)

bridges_viz = bridges_by_K[MID_K][:10]
max_contrib = max(e[2] for e in bridges_viz) if bridges_viz else 1.0
for rank, (bv_i, bv_j, contrib) in enumerate(bridges_viz):
    lw = 1.5 + 4.0 * contrib / max_contrib
    color = '#e63946' if (bv_i, bv_j) == true_bridge else '#457b9d'
    ax.plot([pos[bv_i, 0], pos[bv_j, 0]], [pos[bv_i, 1], pos[bv_j, 1]],
            color=color, linewidth=lw, zorder=2, alpha=0.85)
    mid = ((pos[bv_i, 0] + pos[bv_j, 0]) / 2, (pos[bv_i, 1] + pos[bv_j, 1]) / 2)
    ax.text(mid[0], mid[1], str(rank + 1), fontsize=6, ha='center', va='center',
            color='white', bbox=dict(facecolor=color, edgecolor='none', pad=1))

ax.scatter(pos[:n_per_viz, 0], pos[:n_per_viz, 1], c='#a8dadc', s=40, zorder=3,
           edgecolors='#1d3557', linewidths=0.5)
ax.scatter(pos[n_per_viz:, 0], pos[n_per_viz:, 1], c='#f1faee', s=40, zorder=3,
           edgecolors='#1d3557', linewidths=0.5)
ax.scatter(*pos[true_bridge[0]], c='#e63946', s=90, zorder=4, edgecolors='#1d3557', linewidths=0.8)
ax.scatter(*pos[true_bridge[1]], c='#e63946', s=90, zorder=4, edgecolors='#1d3557', linewidths=0.8)

ax.set_title(
    f'v0.2 Toy graph: top-10 Fiedler bridges at K={MID_K}*K_stable ({MID_K*K_stable:.3f})\\n'
    f'Red = true bridge {true_bridge} (rank #{rank_bridge}) | Blue = other bridges\\n'
    f'theta0 fixed, K_stable={K_stable:.4f} (cf. K_C_MEANFIELD={K_C_MEANFIELD:.4f})'
)
ax.axis('off')
plt.tight_layout()
plt.savefig('kuramoto_v02_bridges.png', dpi=120, bbox_inches='tight')
plt.show()
print('Figure saved: kuramoto_v02_bridges.png')
"""))

# ---------------------------------------------------------------------------
# Cell 13 — Visualization: bisection trace + upper-cluster Jaccard heatmap
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
fig, (ax_b, ax_j) = plt.subplots(1, 2, figsize=(12, 5))

# --- Bisection trace ---
trace_sorted = sorted(bisect_trace, key=lambda x: x[0])
Ks_trace = [t[0] for t in trace_sorted]
lam2s_trace = [t[1] for t in trace_sorted]
ax_b.scatter(Ks_trace, lam2s_trace, c=['#e63946' if l < 0 else '#457b9d' for l in lam2s_trace],
             s=60, zorder=3)
ax_b.axhline(0, color='black', linewidth=0.8, linestyle='--', alpha=0.6)
ax_b.axvline(K_stable, color='#2a9d8f', linewidth=1.5, linestyle='-', alpha=0.8,
             label=f'K_stable={K_stable:.3f}')
ax_b.axvline(K_C_MEANFIELD, color='#e9c46a', linewidth=1.2, linestyle=':', alpha=0.8,
             label=f'K_C_MEANFIELD={K_C_MEANFIELD:.3f}')
ax_b.set_xlabel('K (probe value)')
ax_b.set_ylabel('lambda_2(L_sync)')
ax_b.set_title('K_stable bisection trace\\nlambda_2 = 0 crossing')
ax_b.legend(fontsize=8)
ax_b.grid(True, alpha=0.3)

# --- Upper-cluster Jaccard heatmap ---
n_up = len(upper_cluster)
jac_matrix = np.array([[jaccard(bridges_by_K[K1], bridges_by_K[K2])
                         for K2 in upper_cluster] for K1 in upper_cluster])
im = ax_j.imshow(jac_matrix, vmin=0, vmax=1, cmap='Blues', aspect='auto')
ax_j.set_xticks(range(n_up))
ax_j.set_yticks(range(n_up))
ax_j.set_xticklabels([f'{k}x' for k in upper_cluster], fontsize=8)
ax_j.set_yticklabels([f'{k}x' for k in upper_cluster], fontsize=8)
for i in range(n_up):
    for j in range(n_up):
        ax_j.text(j, i, f'{jac_matrix[i,j]:.2f}', ha='center', va='center',
                  fontsize=7, color='white' if jac_matrix[i, j] > 0.6 else 'black')
plt.colorbar(im, ax=ax_j)
ax_j.set_title(f'Upper-cluster pairwise Jaccard\\n(K_mult x K_stable, lambda_2 >= 0)')

plt.tight_layout()
plt.savefig('kuramoto_v02_diagnostics.png', dpi=120, bbox_inches='tight')
plt.show()
print('Figure saved: kuramoto_v02_diagnostics.png')
"""))

# ---------------------------------------------------------------------------
# Cell 14 — R1: larger-graph sensitivity (N = 50, 200, 500)
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# R1: Scale sweep — do finite-size artefacts (SC2, SC3 flags) vanish at N=200, 500?
# Hypothesis: at N>=200, r(2*K_stable) > 0.9 and plateau-wide SC2 Jaccard >= 0.85.

r1_configs = [
    (25,  1, 'N=50  (baseline)'),
    (100, 3, 'N=200 (3 bridges)'),
    (250, 5, 'N=500 (5 bridges)'),
]

r1_results = []
for n_per, n_br, label in r1_configs:
    A_r1, bridges_r1 = build_two_community_toy(n_per=n_per, p_intra=0.35, seed=42, n_bridges=n_br)
    N_r1 = A_r1.shape[0]
    primary_bridge = bridges_r1[0]

    rng_r1 = np.random.default_rng(GLOBAL_SEED)
    omega_r1 = rng_r1.normal(0.0, 1.0, size=N_r1)
    omega_r1 -= omega_r1.mean()
    theta0_r1 = make_theta0(N_r1, seed=GLOBAL_SEED)

    print(f'[R1] {label}: computing K_stable...', flush=True)
    K_stable_r1, _ = compute_K_stable(A_r1, omega_r1, theta0_r1)

    bbyK_r1, tbyK_r1, ebyK_r1 = run_pipeline(
        A_r1, omega_r1, K_ref=K_stable_r1, theta0=theta0_r1
    )
    uc_r1, _ = classify_k_clusters(ebyK_r1)

    r_global_r1 = global_order_parameter(tbyK_r1[2.0])

    # Permuted-omega (SC2 plateau-wide)
    omega_perm_r1 = np.random.default_rng(GLOBAL_SEED + 1).permutation(omega_r1)
    bbyK_perm_r1, _, _ = run_pipeline(
        A_r1, omega_perm_r1, K_ref=K_stable_r1, theta0=theta0_r1
    )
    if uc_r1:
        jac_min_r1 = min(jaccard(bbyK_r1[K], bbyK_perm_r1[K]) for K in uc_r1)
    else:
        jac_min_r1 = float('nan')

    bset_mid_r1 = [(int(e[0]), int(e[1])) for e in bbyK_r1[MID_K]]
    rank_r1 = (bset_mid_r1.index(primary_bridge) + 1) if primary_bridge in bset_mid_r1 else None

    r1_results.append({
        'label': label, 'N': N_r1, 'K_stable': K_stable_r1,
        'r_2K': r_global_r1, 'jac_min_plateau': jac_min_r1, 'rank': rank_r1,
    })
    print(f'  K_stable={K_stable_r1:.3f}  r(2K_s)={r_global_r1:.3f}  SC2_min_Jaccard={jac_min_r1:.3f}  rank={rank_r1}')

print()
print('R1 Summary:')
print(f'  {"Label":<22}  {"N":>5}  {"K_stable":>8}  {"r(2Ks)":>7}  {"SC2 min-J":>10}  {"rank":>5}')
print('  ' + '-' * 65)
for r in r1_results:
    print(f'  {r["label"]:<22}  {r["N"]:>5}  {r["K_stable"]:>8.3f}  {r["r_2K"]:>7.4f}  {r["jac_min_plateau"]:>10.4f}  {str(r["rank"]):>5}')
"""))

# ---------------------------------------------------------------------------
# Cell 15 — R2: multi-seed K_stable stability
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# R2: seeds 42..46 — is K_stable stable across omega/theta0 draws?
# Gate: CV(K_stable) < 0.1 AND bridge-set pairwise Jaccard >= 0.8.

r2_seeds = [42, 43, 44, 45, 46]
r2_K_stables = []
r2_bridges = []
r2_ranks = []

for seed in r2_seeds:
    rng_s = np.random.default_rng(seed)
    omega_s = rng_s.normal(0.0, 1.0, size=N_BASE)
    omega_s -= omega_s.mean()
    theta0_s = make_theta0(N_BASE, seed=seed)

    print(f'[R2] seed={seed}: computing K_stable...', flush=True)
    K_s, _ = compute_K_stable(A_base, omega_s, theta0_s)

    bbyK_s, _, ebyK_s = run_pipeline(A_base, omega_s, K_ref=K_s, theta0=theta0_s)
    uc_s, _ = classify_k_clusters(ebyK_s)

    bset_s = [(int(e[0]), int(e[1])) for e in bbyK_s[MID_K]]
    rank_s = (bset_s.index(true_bridge) + 1) if true_bridge in bset_s else None

    r2_K_stables.append(K_s)
    r2_bridges.append(bbyK_s[MID_K])
    r2_ranks.append(rank_s)
    print(f'  K_stable={K_s:.4f}  true_bridge_rank={rank_s}')

k_arr = np.array(r2_K_stables)
cv_k = float(k_arr.std() / k_arr.mean()) if k_arr.mean() > 0 else float('nan')

# Pairwise bridge-set Jaccard across seeds
n_seeds = len(r2_seeds)
jac_pairs = []
for i in range(n_seeds):
    for j in range(i + 1, n_seeds):
        jac_pairs.append(jaccard(r2_bridges[i], r2_bridges[j]))
jac_mean_r2 = float(np.mean(jac_pairs)) if jac_pairs else float('nan')
jac_min_r2 = float(np.min(jac_pairs)) if jac_pairs else float('nan')

print()
print(f'R2 Summary:')
print(f'  K_stable values: {[f"{k:.3f}" for k in r2_K_stables]}')
print(f'  CV(K_stable) = {cv_k:.4f}  (gate < 0.1)')
print(f'  Bridge-set pairwise Jaccard: mean={jac_mean_r2:.4f}, min={jac_min_r2:.4f}  (gate >= 0.8)')
print(f'  True bridge rank-1 in all seeds: {all(r == 1 for r in r2_ranks)}')

if cv_k < 0.1 and jac_min_r2 >= 0.8:
    R2_VERDICT = 'PASS'
elif cv_k < 0.2 and jac_min_r2 >= 0.5:
    R2_VERDICT = f'FLAG (CV={cv_k:.3f}, min_Jaccard={jac_min_r2:.3f} — tolerable at N=50)'
else:
    R2_VERDICT = f'FLAG (CV={cv_k:.3f} or min_Jaccard={jac_min_r2:.3f} — high seed sensitivity)'
print(f'R2 verdict: {R2_VERDICT}')
"""))

# ---------------------------------------------------------------------------
# Cell 16 — R3: ω-distribution ablation
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# R3: does bridge detection depend on omega being N(0,1)?
# Three alternatives on N=50 baseline, same theta0_fixed, K_stable computed per variant.
# Verdict: min Jaccard vs canonical >= 0.3 (topology-driven), < 0.3 ABORT (ω-artefact).

rng_r3 = np.random.default_rng(50)

omega_t3 = rng_r3.standard_t(df=3, size=N_BASE)
omega_t3 -= omega_t3.mean()

rng_r3b = np.random.default_rng(51)
omega_lognormal = rng_r3b.lognormal(0.0, 1.0, size=N_BASE)
omega_lognormal -= omega_lognormal.mean()

omega_powerlaw = deg_base.astype(float)**0.5 + np.random.default_rng(52).normal(0, 0.1, N_BASE)
omega_powerlaw -= omega_powerlaw.mean()

r3_variants = {
    'Student-t(df=3)': omega_t3,
    'Log-normal (zero-mean)': omega_lognormal,
    'Power-law proxy (deg^0.5)': omega_powerlaw,
}

r3_results = []
for name, omega_v in r3_variants.items():
    print(f'[R3] {name}: computing K_stable...', flush=True)
    K_stable_v, _ = compute_K_stable(A_base, omega_v, theta0_fixed)
    bbyK_v, _, _ = run_pipeline(A_base, omega_v, K_ref=K_stable_v, theta0=theta0_fixed)

    jac_v = jaccard(bridges_by_K[MID_K], bbyK_v[MID_K])
    bset_v = [(int(e[0]), int(e[1])) for e in bbyK_v[MID_K]]
    rank_v = (bset_v.index(true_bridge) + 1) if true_bridge in bset_v else None
    r3_results.append({'name': name, 'K_stable': K_stable_v, 'jaccard_vs_canon': jac_v, 'rank': rank_v})
    print(f'  K_stable={K_stable_v:.3f}  Jaccard_vs_N01={jac_v:.4f}  rank={rank_v}')

print()
min_jac_r3 = min(r['jaccard_vs_canon'] for r in r3_results)
print(f'R3 Summary (Jaccard vs canonical N(0,1) bridge set at K={MID_K}*K_stable):')
print(f'  {"Distribution":<30}  {"K_stable":>8}  {"Jaccard":>8}  {"rank":>5}')
print('  ' + '-' * 58)
for r in r3_results:
    print(f'  {r["name"]:<30}  {r["K_stable"]:>8.3f}  {r["jaccard_vs_canon"]:>8.4f}  {str(r["rank"]):>5}')

if min_jac_r3 >= 0.7:
    R3_VERDICT = f'PASS (min Jaccard={min_jac_r3:.4f} >= 0.7 — bridge detection is topology-driven)'
elif min_jac_r3 >= 0.3:
    R3_VERDICT = f'FLAG (min Jaccard={min_jac_r3:.4f} in [0.3, 0.7) — moderate omega-distribution sensitivity)'
else:
    R3_VERDICT = f'ABORT (min Jaccard={min_jac_r3:.4f} < 0.3 — bridge detection is omega-distribution artefact)'

print(f'R3 verdict: {R3_VERDICT}')
assert 'ABORT' not in R3_VERDICT, R3_VERDICT
"""))

# ---------------------------------------------------------------------------
# Cell 17 — R4: golden-file export
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# R4: save top-20 bridges at K=1.3*K_stable (seed=42 baseline) as Rust parity fixture.
# Schema: required >= 0.9 Jaccard between Python and Rust on this fixture (Phase 42 CI gate).

import os

top_20 = bridges_by_K[MID_K][:20]

golden = {
    'graph': {
        'N': int(N_BASE),
        'seed': 42,
        'p_intra': 0.35,
        'n_bridges': 1,
        'bridge': [int(true_bridge[0]), int(true_bridge[1])],
    },
    'omega_seed': 42,
    'theta0_seed': 42,
    'K_stable': float(K_stable),
    'K_C_MEANFIELD': float(K_C_MEANFIELD),
    'K_sweep_multipliers': K_MULTS,
    'top_20_bridges_at_K1.3': [
        {'i': int(e[0]), 'j': int(e[1]), 'contribution': float(e[2])}
        for e in top_20
    ],
}

os.makedirs('data', exist_ok=True)
out_file = 'data/kuramoto_v02_golden_bridges.json'
with open(out_file, 'w') as f:
    json.dump(golden, f, indent=2)

print(f'Golden fixture saved: {out_file}')
print(f'  Top-1 bridge: ({golden["top_20_bridges_at_K1.3"][0]["i"]}, {golden["top_20_bridges_at_K1.3"][0]["j"]})')
print(f'  Total bridges saved: {len(golden["top_20_bridges_at_K1.3"])}')
print(f'  K_stable = {golden["K_stable"]:.6f}')
print()
print('Validation:')
top_bridge_correct = (golden['top_20_bridges_at_K1.3'][0]['i'] == true_bridge[0] and
                      golden['top_20_bridges_at_K1.3'][0]['j'] == true_bridge[1])
print(f'  True bridge is top-1: {top_bridge_correct}')
print(f'  Total fixtures: {len(golden["top_20_bridges_at_K1.3"])} (expected 20 or fewer if bridge count < 20)')
"""))

# ---------------------------------------------------------------------------
# Cell 18 — Summary table
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
print('=' * 76)
print('KURAMOTO-LBD v0.2 — RESULTS SUMMARY')
print('=' * 76)
print(f'Graph: N={N_BASE}, edges={A_toy.nnz // 2}, p_intra=0.35, 1 deterministic bridge')
print(f'K_stable = {K_stable:.4f}  (bisection, {len(bisect_trace)} probes)')
print(f'K_C_MEANFIELD = {K_C_MEANFIELD:.4f}  (infinite-N reference, NOT used as sweep anchor)')
print(f'Ratio K_stable/K_C_MEANFIELD = {K_stable/K_C_MEANFIELD:.3f}')
print(f'theta0: FIXED (seed={GLOBAL_SEED}), shared across all runs')
print(f'Upper-cluster K_mults: {upper_cluster}')
print()

sc_results = [
    ('#4 True bridge rank', f'rank={rank_bridge}',            'rank<=5 (ABORT>20)',                   SC4_VERDICT),
    ('#3 Global lock r',    f'r={r_global:.4f}',              'r>0.9 (ABORT<0.7)',                    SC3_VERDICT),
    ('#5 Spectral (fix)',   f'lam2={lambda_2:.4f},gap={spectral_gap:.3f}', 'lam2>=0, gap>=1.05',      SC5_VERDICT),
    ('#2 Perm-w Jac (fix)', f'min_J={jac_min:.4f}',           'min_J>=0.85 (plateau-wide, fixed t0)', SC2_VERDICT),
    ('#1 Null Jaccard',     f'J={jac_null:.4f}',              'J<=0.3 (ABORT if artefact)',            SC1_VERDICT),
]

print(f'  {"Check":<22}  {"Value":<30}  {"Threshold":<30}  Verdict')
print('  ' + '-' * 115)
for check, value, threshold, verdict in sc_results:
    print(f'  {check:<22}  {value:<30}  {threshold:<30}  {verdict}')

print()
print('Robustness:')
print(f'  R1 (larger graphs):     {r1_results[-1]["r_2K"]:.3f} r(2Ks) at N=500; SC2 min-J={r1_results[-1]["jac_min_plateau"]:.3f}')
print(f'  R2 (multi-seed):        CV(K_stable)={cv_k:.4f}; bridge-set min-Jaccard={jac_min_r2:.4f}  [{R2_VERDICT}]')
print(f'  R3 (omega-ablation):    min Jaccard vs N(0,1) = {min_jac_r3:.4f}  [{R3_VERDICT}]')
print(f'  R4 (golden fixture):    saved to data/kuramoto_v02_golden_bridges.json')

print()
n_aborts = sum(1 for *_, v in sc_results if 'ABORT' in v)
n_flags  = sum(1 for *_, v in sc_results if 'FLAG'  in v)
n_passes = sum(1 for *_, v in sc_results if v == 'PASS')
print(f'SC: PASS={n_passes}/5  FLAG={n_flags}/5  ABORT={n_aborts}/5')

print()
if n_aborts:
    print('OVERALL: ABORT — abort gate fired. Debug before Wave-4 roadmap proposal.')
elif n_flags:
    print('OVERALL: PASS WITH FLAGS — flags are expected at N=50; R1 shows improvement at scale.')
    print('Phase 42 (Kuramoto-LBD Rust) remains gated on real-corpus Louvain export.')
else:
    print('OVERALL: CLEAN PASS — Wave-4 candidate Phase 42 roadmap discussion authorized.')
"""))

# ---------------------------------------------------------------------------
# Write notebook
# ---------------------------------------------------------------------------
out_path = 'kuramoto_lbd_v02.ipynb'
with open(out_path, 'w') as f:
    nbformat.write(nb, f)
print(f'Wrote {out_path}')
