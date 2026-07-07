"""Generate kuramoto_lbd_v01.ipynb via nbformat to avoid JSON quoting issues."""
import nbformat

nb = nbformat.v4.new_notebook()

# ---------------------------------------------------------------------------
# Cell 1 — Markdown header
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_markdown_cell("""\
# Kuramoto-LBD Prototype v0.1 — Toy-Graph Sanity Pass

**Pre-registration:** `wiki/analyses/kuramoto-research-synergy-integration.md`
section "Python notebook prototype spec"

**Scope (this run):** Five sanity checks on a 50-node two-community synthetic graph.
No research-synergy Louvain export. No Feynman 10-pair benchmark.
Goal: prove the pipeline (ODE integration → sync Laplacian → Fiedler bridges) is
bug-free before any Rust port.

**Gate:** All ABORT checks must pass. FLAG checks are noted but do not block the
Wave-4 roadmap discussion.
"""))

# ---------------------------------------------------------------------------
# Cell 2 — Imports & seed
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
import numpy as np
from scipy.integrate import solve_ivp
from scipy.sparse import csr_matrix, diags
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

GLOBAL_SEED = 42
rng = np.random.default_rng(GLOBAL_SEED)

K_MULTS = [0.5, 0.7, 1.0, 1.3, 1.5, 2.0, 3.0]
# K_c for Kuramoto with N(0,1) frequencies (infinite-N mean-field):
# K_c = 2 / (pi * g(0)), g(0) = 1/sqrt(2*pi)  =>  K_c = 2*sqrt(2*pi)/pi ≈ 1.5958
K_C = 2.0 * np.sqrt(2.0 * np.pi) / np.pi
print(f'K_c (N(0,1) mean-field) = {K_C:.4f}')
"""))

# ---------------------------------------------------------------------------
# Cell 3 — Toy graph builder
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
def build_two_community_toy(n_per=25, p_intra=0.35, seed=42):
    \"\"\"50-node graph: two ER communities + one deterministic bridge edge.
    Returns: A (csr, N x N), true_bridge=(bridge_i, bridge_j)
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

    # Single deterministic bridge between last node of A and first of B
    bridge_i, bridge_j = n_per - 1, n_per
    rows += [bridge_i, bridge_j]
    cols += [bridge_j, bridge_i]

    data = np.ones(len(rows))
    A = csr_matrix((data, (rows, cols)), shape=(N, N))
    return A, (bridge_i, bridge_j)


A_toy, true_bridge = build_two_community_toy()
N = A_toy.shape[0]
deg_toy = np.array(A_toy.sum(axis=1)).flatten()
print(f'Graph: N={N}, edges={A_toy.nnz // 2}, mean_deg={deg_toy.mean():.2f}, true_bridge={true_bridge}')
print(f'Bridge node degrees: {int(deg_toy[true_bridge[0]])}, {int(deg_toy[true_bridge[1]])}')
"""))

# ---------------------------------------------------------------------------
# Cell 4 — Pipeline functions
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
def kuramoto_rhs(t, theta, K, omega, A_csr, D_inv):
    \"\"\"Sparse-safe RHS: dtheta/dt = omega + K * D^-1 * sum_j A_ij * sin(theta_j - theta_i)
    Identity: sum_j a_ij sin(t_j - t_i) = (A@sin_t)*cos_t - (A@cos_t)*sin_t
    \"\"\"
    cos_t = np.cos(theta)
    sin_t = np.sin(theta)
    coupling = (A_csr @ sin_t) * cos_t - (A_csr @ cos_t) * sin_t
    return omega + K * D_inv * coupling


def global_order_parameter(theta):
    return float(np.abs(np.mean(np.exp(1j * theta))))


def build_sync_laplacian(theta_star, A, K):
    \"\"\"L_sync = -J* where J*_ij = K*cos(theta_j - theta_i)*A_ij.\"\"\"
    i_idx, j_idx = A.nonzero()
    cos_diffs = np.cos(theta_star[j_idx] - theta_star[i_idx])
    n = A.shape[0]
    J_off = csr_matrix((K * cos_diffs, (i_idx, j_idx)), shape=(n, n))
    J_diag = np.array(J_off.sum(axis=1)).flatten()
    return -J_off + diags(J_diag)


def extract_fiedler_bridges(L_sync, A, top_k=100):
    \"\"\"Dense eigh (suitable for N<=500). Returns bridge list (i<j, sorted by -contribution) and eigenvalues.\"\"\"
    eigvals, eigvecs = np.linalg.eigh(L_sync.toarray())
    v2 = eigvecs[:, 1]  # Fiedler vector
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


def run_pipeline(A, omega, K_mults=None, rng_init_seed=0):
    \"\"\"Full K-sweep. Returns bridges_by_K, theta_by_K, eigvals_by_K.\"\"\"
    if K_mults is None:
        K_mults = K_MULTS
    rng_init = np.random.default_rng(rng_init_seed)
    n = A.shape[0]
    deg = np.array(A.sum(axis=1)).flatten()
    D_inv = 1.0 / np.maximum(deg, 1e-12)
    bridges_by_K, theta_by_K, eigvals_by_K = {}, {}, {}
    for K_mult in K_mults:
        K = K_mult * K_C
        theta0 = rng_init.uniform(0, 2 * np.pi, size=n)
        sol = solve_ivp(kuramoto_rhs, (0, 200), theta0,
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


print('Pipeline functions defined.')
"""))

# ---------------------------------------------------------------------------
# Cell 5 — Canonical run
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
rng_canon = np.random.default_rng(GLOBAL_SEED)
omega_canon = rng_canon.normal(0.0, 1.0, size=N)
omega_canon -= omega_canon.mean()  # zero-mean per spec

print('Running canonical K-sweep on toy graph...')
bridges_by_K, theta_by_K, eigvals_by_K = run_pipeline(A_toy, omega_canon)
print('Done.')

MID_K = 1.3  # partial-sync plateau target per spec
print(f'\\nTop-5 Fiedler bridges at K={MID_K}*K_c:')
for rank, (i, j, contrib) in enumerate(bridges_by_K[MID_K][:5], 1):
    flag = ' <-- TRUE BRIDGE' if (i, j) == true_bridge else ''
    print(f'  #{rank}: ({i}, {j})  contrib={contrib:.5f}{flag}')
"""))

# ---------------------------------------------------------------------------
# Cell 6 — Sanity #4 (PRIMARY)
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# SANITY CHECK #4 (PRIMARY): true bridge in top-5 at mid-plateau K
# Spec: top-5 PASS, top-6..20 FLAG, >20 ABORT.

bridge_set_mid = [(int(e[0]), int(e[1])) for e in bridges_by_K[MID_K]]
bridge_i, bridge_j = true_bridge

if (bridge_i, bridge_j) in bridge_set_mid:
    rank_bridge = bridge_set_mid.index((bridge_i, bridge_j)) + 1
else:
    rank_bridge = None

print(f'True bridge {true_bridge} rank at K={MID_K}*K_c: {rank_bridge}')

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
# Cell 7 — Sanity #3
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# SANITY CHECK #3: global lock r(K=2*K_c) > 0.9
# Spec: PASS > 0.9, FLAG 0.7-0.9, ABORT < 0.7

theta_2Kc = theta_by_K[2.0]
r_global = global_order_parameter(theta_2Kc)

print(f'Global order parameter r at K=2.0*K_c: {r_global:.4f}')

if r_global > 0.9:
    SC3_VERDICT = 'PASS'
elif r_global > 0.7:
    SC3_VERDICT = f'FLAG (r={r_global:.4f}, partial lock; sparse bridge may be limiting)'
else:
    SC3_VERDICT = f'ABORT (r={r_global:.4f} < 0.7 — ODE not converging)'

print(f'Sanity #3 verdict: {SC3_VERDICT}')
assert 'ABORT' not in SC3_VERDICT, f'ABORT: r={r_global:.4f}. Check ODE convergence or increase t_span.'
"""))

# ---------------------------------------------------------------------------
# Cell 8 — Sanity #5
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# SANITY CHECK #5: spectral gap lambda_3/lambda_2 >= 1.05
# Spec: FLAG if < 1.05 (Fiedler direction ambiguous)

eigvals_mid = eigvals_by_K[MID_K]
lambda_2 = eigvals_mid[1]
lambda_3 = eigvals_mid[2]
spectral_gap = lambda_3 / max(lambda_2, 1e-12)

print(f'lambda_2 (Fiedler) = {lambda_2:.6f}')
print(f'lambda_3           = {lambda_3:.6f}')
print(f'lambda_3/lambda_2  = {spectral_gap:.4f}')

SC5_VERDICT = 'PASS' if spectral_gap >= 1.05 else f'FLAG (gap={spectral_gap:.4f} < 1.05 — ambiguous Fiedler direction)'
print(f'Sanity #5 verdict: {SC5_VERDICT}')
"""))

# ---------------------------------------------------------------------------
# Cell 9 — Sanity #2
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# SANITY CHECK #2: permuted-omega stability
# Spec: Jaccard >= 0.85 PASS, < 0.5 FLAG (omega-contamination)

rng_perm = np.random.default_rng(GLOBAL_SEED + 1)
omega_permuted = rng_perm.permutation(omega_canon)

print('Running permuted-omega K-sweep...')
bridges_perm, _, _ = run_pipeline(A_toy, omega_permuted, rng_init_seed=GLOBAL_SEED + 1)
print('Done.')

jac_perm = jaccard(bridges_by_K[MID_K], bridges_perm[MID_K])
print(f'Jaccard(canonical, permuted-omega) at K={MID_K}*K_c: {jac_perm:.4f}')

if jac_perm >= 0.85:
    SC2_VERDICT = 'PASS'
elif jac_perm >= 0.5:
    SC2_VERDICT = f'FLAG (Jaccard={jac_perm:.4f}, moderate omega-sensitivity)'
else:
    SC2_VERDICT = f'FLAG (Jaccard={jac_perm:.4f} < 0.5 — high omega-sensitivity, failure mode 4)'

print(f'Sanity #2 verdict: {SC2_VERDICT}')
"""))

# ---------------------------------------------------------------------------
# Cell 10 — Sanity #1
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# SANITY CHECK #1: degree-matched ER null graph
# Adapted: ABORT if null Jaccard with canonical > 0.3 AND canonical true bridge is top-10 in null.
# Rationale: method should detect topology-specific structure, not generic artefacts.

def build_er_null(A, seed=99):
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


A_null = build_er_null(A_toy)
print(f'Null graph: N={A_null.shape[0]}, edges={A_null.nnz // 2}')
print('Running null-graph K-sweep...')
bridges_null, _, _ = run_pipeline(A_null, omega_canon, rng_init_seed=GLOBAL_SEED + 2)
print('Done.')

bi, bj = true_bridge
null_bridge_set = [(int(e[0]), int(e[1])) for e in bridges_null[MID_K]]
true_bridge_in_null_adj = bool(A_null[bi, bj])
rank_in_null = null_bridge_set.index((bi, bj)) + 1 if (bi, bj) in null_bridge_set else None
jac_null = jaccard(bridges_by_K[MID_K], bridges_null[MID_K])

print(f'True bridge {true_bridge} in null adjacency: {true_bridge_in_null_adj}')
print(f'True bridge rank in null Fiedler bridges: {rank_in_null}')
print(f'Jaccard(canonical, null) at K={MID_K}*K_c: {jac_null:.4f}')

is_bad_null = true_bridge_in_null_adj and rank_in_null is not None and rank_in_null <= 10 and jac_null > 0.3

if is_bad_null:
    SC1_VERDICT = f'ABORT (Jaccard={jac_null:.4f} > 0.3 and true bridge rank={rank_in_null} in null — artefact)'
elif jac_null <= 0.3:
    SC1_VERDICT = f'PASS (Jaccard={jac_null:.4f} <= 0.3 — topology-sensitive)'
else:
    SC1_VERDICT = f'FLAG (Jaccard={jac_null:.4f} > 0.3 but bridge absent/low-ranked in null; investigate)'

print(f'Sanity #1 verdict: {SC1_VERDICT}')
assert 'ABORT' not in SC1_VERDICT, f'ABORT: null Jaccard={jac_null:.4f}, rank_in_null={rank_in_null}.'
"""))

# ---------------------------------------------------------------------------
# Cell 11 — Plateau detection
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
# K-plateau: K values where Jaccard with every other K > 0.7 (spec lines 202-208)

plateau = [
    K for K in bridges_by_K
    if all(jaccard(bridges_by_K[K], bridges_by_K[K2]) > 0.7
           for K2 in bridges_by_K if K2 != K)
]

print('K-plateau (strict: Jaccard > 0.7 with ALL other K values):')
if plateau:
    print(f'  Stable K values: {[str(k) + "*K_c" for k in plateau]}')
else:
    print('  No K passes strict plateau criterion (expected on toy graph).')
    print(f'  Fallback: using K={MID_K}*K_c for visualization.')
    plateau = [MID_K]

print()
print('Pairwise Jaccard matrix across K sweep:')
header = '        ' + '  '.join(f'{k:.1f}' for k in K_MULTS)
print(header)
for K1 in K_MULTS:
    row = f'{K1:.1f}:    ' + '  '.join(f'{jaccard(bridges_by_K[K1], bridges_by_K[K2]):.2f}' for K2 in K_MULTS)
    print(row)
"""))

# ---------------------------------------------------------------------------
# Cell 12 — Visualization
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
n_per = N // 2
pos = np.zeros((N, 2))
rng_layout = np.random.default_rng(7)
pos[:n_per, 0] = -1.0 + 0.3 * rng_layout.standard_normal(n_per)
pos[:n_per, 1] = rng_layout.standard_normal(n_per)
pos[n_per:, 0] = +1.0 + 0.3 * rng_layout.standard_normal(n_per)
pos[n_per:, 1] = rng_layout.standard_normal(n_per)

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

ax.scatter(pos[:n_per, 0], pos[:n_per, 1], c='#a8dadc', s=40, zorder=3, edgecolors='#1d3557', linewidths=0.5)
ax.scatter(pos[n_per:, 0], pos[n_per:, 1], c='#f1faee', s=40, zorder=3, edgecolors='#1d3557', linewidths=0.5)
ax.scatter(*pos[true_bridge[0]], c='#e63946', s=90, zorder=4, edgecolors='#1d3557', linewidths=0.8)
ax.scatter(*pos[true_bridge[1]], c='#e63946', s=90, zorder=4, edgecolors='#1d3557', linewidths=0.8)

ax.set_title(
    f'Toy graph: top-10 Fiedler bridges at K={MID_K}*K_c\\n'
    f'Red = true bridge {true_bridge} (rank #{rank_bridge}) | Blue = other bridges\\n'
    f'Edge width proportional to Fiedler contribution'
)
ax.axis('off')
plt.tight_layout()
plt.savefig('kuramoto_toy_bridges.png', dpi=120, bbox_inches='tight')
plt.show()
print('Figure saved to kuramoto_toy_bridges.png')
"""))

# ---------------------------------------------------------------------------
# Cell 13 — Summary table
# ---------------------------------------------------------------------------
nb.cells.append(nbformat.v4.new_code_cell("""\
print('=' * 72)
print('KURAMOTO-LBD v0.1 — TOY-GRAPH SANITY PASS RESULTS')
print('=' * 72)
print(f'Graph: N={N}, edges={A_toy.nnz // 2}, p_intra=0.35, 1 deterministic bridge')
print(f'K_c (N(0,1) mean-field) = {K_C:.4f}  |  Mid-plateau K = {MID_K}*K_c')
print(f'True bridge: {true_bridge}')
print()

results = [
    ('#4 True bridge rank', f'rank={rank_bridge}',        'rank<=5 (ABORT>20)',          SC4_VERDICT),
    ('#3 Global lock r',    f'r={r_global:.4f}',           'r>0.9 (ABORT<0.7)',           SC3_VERDICT),
    ('#5 Spectral gap',     f'lam3/lam2={spectral_gap:.4f}', 'ratio>=1.05',              SC5_VERDICT),
    ('#2 Permuted-w Jac',   f'J={jac_perm:.4f}',           'J>=0.85',                    SC2_VERDICT),
    ('#1 Null Jaccard',     f'J={jac_null:.4f}',           'J<=0.3 (ABORT if artefact)', SC1_VERDICT),
]

print(f'  {"Check":<20}  {"Value":<22}  {"Threshold":<28}  Verdict')
print('  ' + '-' * 100)
for check, value, threshold, verdict in results:
    print(f'  {check:<20}  {value:<22}  {threshold:<28}  {verdict}')

print()
n_aborts = sum(1 for *_, v in results if 'ABORT' in v)
n_flags  = sum(1 for *_, v in results if 'FLAG'  in v)
n_passes = sum(1 for *_, v in results if v == 'PASS')
print(f'PASS: {n_passes}/5   FLAG: {n_flags}/5   ABORT: {n_aborts}/5')
print()

if n_aborts:
    print('OVERALL: ABORT — one or more abort gates fired. Debug before Wave-4 roadmap proposal.')
elif n_flags:
    print('OVERALL: PASS WITH FLAGS — review flagged items, then proceed to Wave-4 discussion.')
else:
    print('OVERALL: CLEAN PASS — unlocks Wave-4 roadmap candidate (Phase 42 Kuramoto-LBD).')
"""))

# ---------------------------------------------------------------------------
# Write notebook
# ---------------------------------------------------------------------------
out_path = 'kuramoto_lbd_v01.ipynb'
with open(out_path, 'w') as f:
    nbformat.write(nb, f)
print(f'Wrote {out_path}')
