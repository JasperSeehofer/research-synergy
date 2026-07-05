# Cross-Field Transfer — Definitive Brainstorm Report

*Research Synergy (ReSyn) / Dynamical-LBD thread, post-kill. Seed for the next chapter.*
*Produced 2026-07-05 by a fan-out ideation workflow (32 agents, ~2.2M tokens: 16 lenses → 78 raw
ideas → 13 clustered directions → adversarial critique → synthesis → completeness critique).
Grounded against the live stack: `resyn-core/src/llm/{claude,prompt,gap_prompt}.rs`,
`resyn-core/src/gap_analysis/abc_bridge.rs`, `resyn-core/src/datamodels/community_graph.rs`, and the
valid testbed at `professional-vault/prototypes/data/` (`research_synergy_bridged_fine.json`,
`cross_bridges_ground_truth.json`, `feynman_10pair_papers.json`, `modern_lbd_pairs.json`,
`concept_aliases.json`).*

---

## 1. Reframe

The goal is unchanged: **STRONG form** — given a stuck problem in field A, transfer a
derivation/method/result from a field B where an analogous problem was already solved; **WEAK
form** — surface cross-field synergies researchers aren't aware of. What changed is where we now
know the signal is *not*. Six phases (29→34) proved cleanly and mechanistically that **purely
structural / spectral / dynamical operators over citation + c-TF-IDF community graphs do not recover
known cross-domain analogies** — Kuramoto's single global Fiedler cut structurally puts every
benchmark pair on the same side (recall@10 = 0 on a fully valid, bridge-containing, synchronized
corpus), and sheaf frustration ranked the pairs #69–218. The analogy signal lives in the **ideas /
equations / mechanisms / problem-structure** — the semantic-conceptual content of the papers — not
in graph topology. The entire next chapter must operate on *that* layer. Two corollaries the
post-mortem forces on us: (a) the bar to beat is **the brute-force LLM community-pair baseline
(EXP-RS-10)** — not the dead dynamical methods — and that number is *currently un-run on the valid
testbed*, so establishing it is job zero; (b) `recall@k` on ~4–5 famous Feynman pairs is
underpowered **and** leaky (Claude knows Ising↔opinion from pretraining), so the more honest
deliverable is an **auditable, checkable bridge artifact** (an alignment table, a CAS certificate, a
shared mechanism tag) that a fuzzy "is there a bridge?" prompt structurally cannot produce.

---

## 2. Top directions (ranked)

Ranking = impact × feasibility × novelty, gated on two hard filters: **escapes the dynamical trap**
and **has a credible path to beating the brute-force LLM baseline, not just the dead methods.**

| # | Direction | Feas | Nov | Eval | Verdict | Role |
|---|---|---|---|---|---|---|
| 1 | SME over LLM-extracted schemas | 4 | 3 | 4 | promising | Generator (flagship) |
| 2 | Slot-frame problem↔method transfer | 4 | 3 | 4 | promising | Generator (strong-form product) |
| 3 | CAS-verified transfer certificates | 4 | 4 | 4 | risky | Verifier (flagship) |
| 4 | Mechanism-ontology tagging (MethMeSH) | 4 | 3 | 3 | promising | Generator (O(N), scalable) |
| 5 | Equation-skeleton fingerprinting | 3 | 3 | 4 | promising | Generator (literal cash-out) |
| 6 | Citance + eponym mining | 4 | 2 | 4 | promising | Eval/data play |

### 1. Structure-Mapping (SME) over LLM-extracted relational schemas — *flagship generator*

**Idea.** Extend the Claude extractor to emit, per paper, a *typed relational schema*: entities
tagged from a closed role vocabulary (control-parameter, order-parameter, coupling,
conserved-quantity, threshold, …) with all domain nouns alpha-renamed, plus higher-order relations
(CAUSES, UNDERGOES-TRANSITION-AT, CONSERVED-UNDER, COUPLES). Match cross-community pairs with a
classic structure-mapping / typed-VF2 aligner scored by **systematicity** (deep interconnected
relations rewarded, surface attributes get zero credit). The alignment table itself (spin↔opinion,
magnetization↔consensus) is the auditable bridge.

**Why it could work.** It attacks the exact layer the kill-lesson points to, and its central
ablation — **role-typing ON vs OFF** — is a clean, pre-registerable test of the load-bearing claim
(relational structure, not lexical overlap, carries the signal). Built *blind* (single-paper
extraction, symbolic matcher the LLM never sees), it prevents the leakage that plagues end-to-end
LLM scoring.

**Smallest MVP.** 1–2 days, no new infra. ~4–5 evaluable Feynman-pair papers + ~40–90 distractors
from `research_synergy_bridged_fine.json`; extend the Claude prompt (`resyn-core/src/llm/claude.rs`
+ `prompt.rs`) to emit the blind, abstract-only typed schema; ~200 lines of Python SME-lite in
`prototypes/` (networkx VF2 / greedy systematicity score, MAC/FAC role-multiset shortlist).

**Eval.** `recall@10`/`precision@10`; **roles-ON vs roles-OFF** ablation; **head-to-head vs
brute-force EXP-RS-10 on the same set**; score alignment tables against `cross_bridges_ground_truth.json`
`bridge_names`. Run on **both** Feynman *and* held-out `modern_lbd_pairs.json`.

**Novelty.** Gentner SME is 1980s; novelty is the LLM-extracted, role-canonicalized, *blind*
pipeline over a real corpus with a systematicity metric — not the deprecated flat-embedding shortcut.

**Key risk.** Information loss: the schema bottleneck may discard context an end-to-end LLM keeps →
SME *ties* rather than beats on `recall@k`. Mitigation: treat the **alignment-table quality** as a
co-primary deliverable. Secondary: closed role vocab is a stat-mech prior suspiciously matched to an
all-stat-phys benchmark.

### 2. Directed problem↔method transfer via typed slot-frames — *the actual strong-form product*

**Idea.** Reframe LBD from symmetric corpus-scan to **asymmetric, query-driven** method→problem
transfer. Per-paper frame with *separate* slots for problem-structure and for solution-method + its
applicability *preconditions*, domain nouns masked. METHOD library keyed by what each technique
*needs*; PROBLEM index keyed by what each open problem *has*; a hit = method M from B satisfies open
problem P in A, never applied, literatures disjoint → a concrete recipe with variable-by-variable
mapping.

**Why it could work.** Literal strong-form deliverable and *product-aligned* — paste one stuck
problem, get solved analogues. Evaluation as **conditional retrieval** (given A-side, retrieve
B-side) is easier to pre-register than symmetric `recall@k`.

**Smallest MVP.** Skip slot extraction first. On the testbed build two per-community embeddings — a
**method-axis** and an **object/substrate-axis** — and rank cross-community pairs by *method-cosine
HIGH ∧ object-cosine LOW* ("unlikely siblings"). 1–2 day Python notebook, no Rust.

**Eval.** Conditional MRR/`recall@10` vs (a) EXP-RS-10, (b) plain-cosine null, (c) random null.
Ablations: LLM fully removed (bounds leakage); hold out one pair's B-side vocabulary.

**Key risk.** Over-abstraction precision collapse ("everything is optimization"). The MVP is
explicitly designed to detect this before any slot apparatus is built.

### 3. CAS-verified symbolic transfer certificates — *flagship verifier (highest novelty)*

**Idea.** Close the loop to a **machine-checked** transfer. LLM *proposes* a change-of-variables
from a fixed menu (log/Cole-Hopf, Fourier/Laplace, Wick rotation, system-size expansion…) as
executable SymPy; a CAS *disposes* via a tiered check (exact identity → numeric spot-check →
asymptotic-order). On success emit a **transfer certificate** ("your pricing PDE is the heat
equation under u=log S, τ=T−t"). Wrap with an adversarial proposer/skeptic gate (a separate-model
prosecutor attacks the weakest correspondence and projects unused source relations as *candidate
inferences* — the discovery payload).

**Smallest MVP.** One day, no crawl/DB. Hand-encode 3 positive pairs (Black-Scholes↔heat,
replicator↔Lotka-Volterra, diffusion↔free Schrödinger); cold prompt → CAS tiered check. 2 negative
controls that must NOT certify + 1 limit-only pair (master→Fokker-Planck). Pass: ≥2/3 cold-verified,
0/2 false positives.

**Eval.** Decoy-separation + false-positive rate, not `recall@k`. **Precision layer over a separate
generator**, not a discovery engine alone.

**Key risk.** Coverage collapse + circularity: strict identity certifies only exact transforms
(already-textbook cases); interesting analogies hold only in a limit. Deploy as re-ranker/confirmer,
never generator.

### 4. Mechanism-ontology tagging (MethMeSH) — *the O(N) scalable ABC refactor*

**Idea.** Replace the word basis with a curated ~50–150 field-agnostic mechanism/archetype
vocabulary (mean-field/Ising, SIR-compartmental, replicator, reaction-diffusion,
master/Fokker-Planck, percolation-threshold, message-passing/cavity, martingale…). LLM multi-labels
each paper **with an equation-citing evidence snippet**; candidate = two papers in different
communities sharing a *rare* (high-IDF) archetype, ranked signature-similarity **minus**
lexical-similarity. Direct refactor of shipped `abc_bridge.rs` — swaps surface-keyword intermediary
for a canonical mechanism concept — O(N²) pairwise → **O(N) tagging + inverted index**.

**Smallest MVP.** 1–2 days. Freeze a ~40-node vocabulary from Wikidata/nLab *before* looking at
pairs (do NOT seed from the Feynman food_set — leakage). 6 held-out `modern_lbd_pairs.json` sides +
~300 distractors; LLM-tag all ~312 with equation-grounded snippets (LaTeX via
`mcp__gpd-arxiv__download_source`). Report **(a) tagging recall** (both sides shared archetype? if
<4/6, dead) and **(b)** `recall@10`/`precision@10` vs EXP-RS-10 at ≥10× lower LLM cost.

**Key risk.** Fixed-vocabulary granularity bind with no principled escape (coarse floods precision,
bespoke falls outside library) — the c-TF-IDF tension relabeled from words to archetypes. Half the
Feynman benchmark is circular against this ontology.

### 5. Equation-skeleton fingerprinting from arXiv LaTeX — *the most literal cash-out*

**Idea.** Pull arXiv LaTeX, parse governing equations to SymPy ASTs, **canonicalize** (alpha-rename
to role-typed placeholders, sort commutative operands, strip constants, reduce to operator skeleton:
derivative order, Laplacian/advection, bilinear u·v, nonlinearity class, conservation form), hash to
a model bucket + coarse operator-multiset for fuzzy MinHash/tree-edit. Candidate = two papers
colliding on a *rare* canonical skeleton in different communities. Side-by-side equations with
induced symbol correspondence = highest-trust "aha".

**Smallest MVP.** Weekend, no DB. 2–4 arXiv papers/side for the 10 pairs (~40–60), LaTeX via
`mcp__gpd-arxiv__download_source`, extract governing equations, canonicalize, hash. Report **(a)**
parse coverage %, **(b)** how many of 10 pairs collide across categories, **(c)** false-collision
rate vs random control. Automate only if ≥4/10 collide with manual equation selection.

**Key risk.** Coverage collapse on the *other-field* side (econ/bio state models verbally); some
analogies are conceptual not equational (percolation↔epidemics is combinatorial). Scope to
arXiv-source fields; SME/prose fallback elsewhere.

### 6. Citation-context & eponym mining of attested transfers — *fixes the thread's #1 liability*

**Idea.** Mine the corpus for **attested** cross-domain transfers → literature-labeled gold set that
dwarfs the 10-pair benchmark. For each cross-community citation edge, pull the citing sentence, LLM
as *classifier* (not generator): is this an explicit method/derivation transfer, and (source-field,
target-field, transferred-object)? In parallel, alias-aware NER for named models (Ising, Kuramoto,
SIR…) from Wikidata/nLab — a named model spanning distant concepts is a fossil-record analogy that
already travelled.

**Smallest MVP.** A few days. Sample ~150 cross-community edges, extract citing sentence (LaTeX MCP
or `ArxivHTMLDownloader`), classify at temp 0, human-label ~30 for precision. Eponym matcher seeded
from `concept_aliases.json` + ~30 Wikidata/nLab models. Readouts: ≫10 clean attested transfers?
classifier precision >0.8? do Feynman pairs surface (recall sanity)?

**Key risk.** Survivorship + circularity: harvests only *made* bridges; the "unmade-bridge
discovery" inversion collapses back to fuzzy similarity. Ship as gold-set/baseline/benchmark
expansion; gate any discovery-inversion behind a temporal holdout that must beat EXP-RS-10.

---

## 3. Wildcards (high-risk / high-upside)

- **Fields-as-languages embedding alignment.** Per-field concept embeddings; seed orthogonal
  Procrustes with a ~40–60-entry cross-field anchor dictionary (temperature↔noise, free-energy↔ELBO,
  partition-function↔evidence, Hamiltonian↔loss…), retrieve with CSLS. Decoupled metric (held-out
  bilinear-lexicon-induction p@1/@5 vs random-rotation null). **Upside:** the learned dictionary is
  itself a strong-form asset (translates B's derivation into A's terms). **Risk:** likely validates
  *already-known* stat-phys↔ML analogies; surprising bridges are the non-isometric parts a linear
  map won't align. Kill rule: held-out p@5 ≥ 2× null.
- **Intellectual-migration / carrier-flux early warning.** Detect the best-documented transfer
  vehicle — a *person* who moves fields carrying a toolkit (econophysics, RG→ML, spin-glasses→neural
  nets) — from OpenAlex author trajectories, **no LLM for stage 1**. Rising B→A carrier flux flags a
  field-pair as ripe *before* the first dense cross-citation bridge. **Risk:** flux may be
  *coincident* not *leading*; field-pair signal can't rank model-pairs. MVP falsifies the load-bearing
  claim in ~2–4 days at near-zero cost — worth the cheap shot.
- **Rigorous invariant fingerprints as a verifier tier.** Buckingham-Pi dimensionless groups, Lie
  point-symmetry classes, bifurcation normal forms — precision near-1 by construction on the
  DE-expressible subset. Best role = a **certifying tier inside the CAS verifier (#3)**, not
  standalone. Cheap MVP: hand-transcribe 4–5 ODE/PDE pairs + Black-Scholes↔heat control + 10
  negatives; implement Buckingham-Pi (SymPy nullspace) + bifurcation label; kill if pi-groups can't
  separate textbook equations.

---

## 4. Recommended first move

**Establish the baseline and test the flagship generator, head-to-head, in one 2-week package — with
the auditable artifact as a co-primary deliverable.**

Nothing here can be *ranked promising* until the brute-force baseline (**EXP-RS-10, BF-community-pairs
LLM**) has an actual number on the valid testbed — per THREAD.md it is the reverted working baseline
but is **un-run** on `research_synergy_bridged_fine.json`. The bar literally does not exist yet.

**Scope (2 weeks):**
- **Week 1 — build the bar + the generator in parallel.**
  - (a) Run EXP-RS-10 brute-force LLM pairwise community comparison over the testbed (1400 nodes, 34
    communities, 4/4 pairs bridged). Record `recall@10`/`precision@10` + per-pair MRR — the number
    everything is judged against.
  - (b) Extend the Claude extractor to emit the **blind, abstract-only, role-typed schema**; write
    ~200 lines of Python **SME-lite** in `prototypes/`.
- **Week 2 — decisive comparison + ablations.**
  - `recall@10`/`precision@10` for SME on Feynman **and** held-out `modern_lbd_pairs.json`.
  - **Roles-ON vs roles-OFF** ablation (the exact question the kill-lesson raises).
  - Score alignment tables against `cross_bridges_ground_truth.json` `bridge_names`.

**Pre-registered kill/success (lock before running):**
- **Advance** iff SME clears the TF-IDF floor (`BENCH_P10 > 0.15`) AND `recall@10` ≥ brute-force
  EXP-RS-10 AND roles-ON > roles-OFF AND alignment tables match ground-truth bridge names.
- **Pivot (don't kill)** iff SME *ties* brute-force on `recall@k` but alignment-table quality is high:
  deliverable becomes the **auditable bridge artifact**, metric shifts to certified-mapping quality,
  and the CAS verifier (#3) becomes the next build (SME generates → CAS certifies).
- **Kill SME** iff it fails the TF-IDF floor OR roles-ON ≤ roles-OFF (relational structure carried
  nothing) → fall back to slot-frames (#2) or mechanism-ontology (#4).

In parallel, spin up the **citance/eponym harvest (#6)** as a background task — the honest
medium-term fix for every method here is a bigger, leakage-controlled, dated gold set to replace the
10-pair straitjacket.

**Explicitly deprioritized (do not fund first):** recsys bipartite link-prediction (matrix
factorization *is* a spectral operator — the killed family); co-citation-mined eval redesign
(re-imports the structural signal proven not to carry analogy — keep only its dated/leakage-controlled
core, fold into #6); de-domained schema embeddings (the thread's own already-deprecated flat
Feynman-reduction at the embedding tier — SME #1 is the graph-matching version that concept preferred).

---

## 5. Completeness critique (what the report overlooked)

1. **Borrow prior systems AND their bigger gold sets.** Three directions reinvent benchmarked
   systems: Hope & Kittur "Accelerating Innovation Through Analogy Mining" (KDD 2017) already did
   #2's purpose-vs-mechanism split with a real dataset; biomedical LBD's **SemRep/SemMedDB** is a
   mature, validated version of #1's typed relational schema (subject-predicate-object predications,
   decades of eval methodology); **TRIZ** is a human-curated ~40-principle cross-domain inventive
   ontology (a pre-built #4 vocabulary, cross-domain *by construction* — defuses the stat-phys-overfit
   risk). *How:* port Hope-Kittur purpose/mechanism embeddings; adopt SemRep-style predication
   extraction; seed #4 from TRIZ.
2. **Temporal discovery-replay should be the PRIMARY eval, not a footnote.** The infra exists
   (`--published-before`). Train on literature ≤ T, test whether the method ranks a bridge *actually
   made only after T* — the only eval that measures genuine discovery (unmade→made), auto-generates a
   large dated gold set, and (picking transfers attested after the pretraining cutoff) is the cleanest
   defense against leakage. *How:* freeze at 2010/2015, score all candidates, report "time-to-discovery
   lift" vs baseline.
3. **The strong-form goal is inherently human-in-the-loop — that changes the metric.** Real success =
   "~20 candidates/week where 2–3 are worth an expert's time" (precision-at-triage, sidesteps recall@k);
   mixed-initiative refinement beats an autonomous scan; expert novelty/usefulness ratings are the only
   honest measure of the WEAK form (recall@k on known pairs structurally *cannot* measure surfacing
   something new). *How:* one-query demo (A-problem → ranked B-candidates + alignment table) + a small
   blinded expert study.
4. **The kill lesson was over-generalized — untested quadrant: topology as a PRECISION FILTER over a
   semantic generator.** The post-mortem proved global spectral operators fail as *generators*, not
   that graph structure is useless as a *constraint*. Swanson's disjointness requirement (A and C
   literatures genuinely disconnected — no co-citation/shared-authors) is exactly what topology
   verifies well and semantics verify poorly. *How:* gate any semantic generator's top candidates on
   graph-distance / co-citation-absence / author-disjointness; measure precision lift.
5. **Every benchmark pair is a TRUE analogy — the eval never measures the spurious-analogy rate,
   which is what historically kills LBD.** A method with high recall@10 that also fires on 10⁴ garbage
   pairs is worthless; only #3 has decoys. Need *near-miss* decoys (same equation skeleton / shared
   archetype but semantically invalid) — exactly what #4/#5 over-fire on. *How:* build a decoy pool,
   report decoy-separation/ROC + expert-adjudicated precision on the top-k the system is MOST confident
   about.
6. **HyDE-style generative query expansion sidesteps the schema-bottleneck (SME's own top risk).**
   Don't extract a schema — have the LLM *hallucinate the ideal solved analogue* ("write the abstract
   of a paper from another field that already solved this mechanism, domain nouns generic"), embed that
   hypothetical, retrieve real B-papers against it (Hypothetical Document Embeddings). Keeps the LLM's
   full richness while making generation tractable as retrieval; directly upgrades #2's method-axis MVP.

**Framing meta-note:** the report treats the six directions as *competitors* racing the O(N²)
baseline, but the production system is almost certainly a **cascade** (cheap O(N) tagging #4 →
mid-tier SME #1 → CAS certifier #3). Under that reading the right generator metric is **recall@100**
(the funnel's recall ceiling — a verifier can't recover a pair it never sees), not recall@10, and the
head-to-head gate should evaluate the assembled cascade's **cost-per-true-bridge**, not each stage in
isolation.
