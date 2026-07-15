# EXP-RS-20 — Generate→Verify Cascade: generation-based cross-field retrieval with computational verification (design line #3)

- **Status**: pre-registration DRAFT (predictions to be LOCKED before any run)
- **Provenance**: Direct follow-up to EXP-RS-19 (HyDE-Bridge). EXP-RS-19 proved the generation mechanism works (pair04 recovered rank 17→4 via epidemiology hypothetical matching the real epidemics paper) and tied the LLM baseline at K=1 (recall@10 = 0.60), but the pinned K=5 max-pool headline was KILLED by distractor inflation (monotonic: K1=0.60, K3=0.40, K5=0.20). The signal survives the lexical comparator but is throttled by aggregation noise. The thread's recommended forward — and the convergence point of both C-37's escalation branch and the design's PIVOT — is a **generate→VERIFY cascade**: keep HyDE's recall stage (proven to convert latent cross-field equivalences into retrievable tokens) and add a VERIFY stage (an LLM/computational check that audits each proposed transfer for method/object coherence and prunes distractors).

## Hypothesis (H-RS-analogy-cascade)

A cross-field analogy = SAME method, DIFFERENT object; it is recoverable by a two-stage cascade:
1. **GENERATE** (HyDE): from the query alone, generate hypothetical abstracts re-expressing its method in other fields' native vocab (proven in EXP-RS-19 to recover cross-vocabulary pairs the lexical comparator misses)
2. **VERIFY** (LLM/CAS): audit each proposed transfer card for method/object coherence and prune distractors

The cascade beats the brute-force LLM baseline (recall@10 = 0.60) on the shared Feynman pairs AND holds up on the modern held-out set where the LLM's pretraining advantage is neutralized.

**Discriminating experiment**: Run the full generate→verify cascade on the Feynman MVP and modern held-out corpora, with the HyDE recall stage frozen (reuse EXP-RS-19's blind prompt + scorer + generations) and a new blind VERIFY stage (benchmark-agnostic prompt, hashed before use). Score vs the 0.60 bar + nulls + the auditable transfer-card check vs `bridge_names`.

## Predictions (to LOCK before any run)

**P1 (cheap forward gate — decisive, run FIRST)**:
- Feynman forward recall@10 ≥ 3/5 AND recovers ≥1 pair the C-17 null misses (pair01/04/06)
- pair04 (percolation→epidemics) recovered into forward top-10
- FAIL → KILL before any modern/reverse/ablation spend

**P2 (beats the lexical floor)**:
- Cascade forward recall@10 > 0.40 (C-17 null) on Feynman
- If it can't beat free lexical retrieval, the verify stage added only noise → cascade premise refuted

**P3 (the bar — HONEST TIE, not beat)**:
- Cascade Feynman recall@10 ≥ 0.60 (tie-or-narrow-beat the leaky LLM)
- Modern recall@10 ≥ the frozen modern bar AND ≥ 0.833 (no regression vs the modern null)
- Modal outcome = TIE at 0.60 (nobody recovers pair01/06), so the recall number is NOT the deliverable

**P4 (verify stage lifts precision)**:
- Cascade recall@10 > HyDE-alone (K=5 max-pool) on BOTH corpora
- The verify stage prunes ≥2 distractors per query on average (auditable via the pruned-card log)

**P5 (auditable artifact — THE PRIMARY DELIVERABLE)**:
- For ≥3/5 recovered Feynman pairs, the winning transfer card's {method_core ∪ generic_object} tokens match `bridge_names` under the C-36 objective rule
- A random bridge_name control does NOT match
- The verify stage's audit log shows ≥1 method/object coherence check per recovered pair

**GATE**:
- **ADVANCE** iff STRICT DOUBLE-BEAT — cascade strictly beats BOTH the Feynman 0.60 bar (≥0.80) AND the frozen modern bar → generation+verification is a genuine generator; formalize + scale
- **PIVOT (expected)** iff cascade TIES the incumbent (Feynman ~0.60; modern ≥ frozen bar ∧ ≥ 0.833) AND P4/P5 hold → bank the artifact pipeline + cheap O(#queries) retriever; next build = verify-stage tuning (LLM vs CAS, prompt ablations)
- **KILL** iff P1 fails (even the strongest cascade has no bridging signal) → the lexical-retrieval line is exhausted; escalate OUT of lexical intermediates (real semantic-embedding substrate or pure LLM-judge cascade), do NOT run another TF-IDF variant

## Setup

**Reused (frozen residue from EXP-RS-19)**:
- Feynman MVP corpus (C-14) + modern held-out (C-24)
- Blind HyDE prompt (`prototypes/hyde_prompt.md`, SHA-256 `...`) + scorer (`hyde_score.py`)
- 5 frozen Feynman side_a generations (`data/hyde_hypotheticals_feynman.json`, SHA-256 `...`)
- Verified pair04 transfer card (percolation→epidemiology)
- C-19 retrieval metric, sme_lite scorer, C-17 null (Feynman 0.40, modern 0.833)
- C-20 baseline (Feynman 0.60) + C-35 modern bar (frozen before modern scoring)

**NEW (verify stage)**:
- **Blind VERIFY prompt** (`prototypes/verify_prompt.md`): benchmark-agnostic, authored by a no-benchmark subagent, non-benchmark examples, SHA-256 committed BEFORE any verification
- **Inputs**: per candidate pair, the query paper's {title, abstract}, the winning hypothetical's {target_field, generic_object, abstract}, and the candidate paper's {title, abstract}
- **Output**: `{method_coherence: bool, object_difference: bool, rationale: str}` — a binary audit of whether the proposed transfer is method-coherent and object-different
- **Pruning**: candidates with `method_coherence = false` are demoted to rank ∞ (effectively removed from the top-k)
- **Frozen before scoring**: the verify prompt SHA-256 + the first 5 Feynman verify logs are committed before any full-corpus scoring

**Conventions**:
- Reuse C-14, C-17, C-19, C-20, C-24, C-31..C-37
- Propose C-38..C-40 (see below)

## Metric

- **Primary**: recall@{1,5,10} + MRR on Feynman AND modern held-out, vs:
  - Feynman 0.60 bar + 0.40 null
  - Modern frozen bar + 0.833 null
- **Secondary**:
  - Auditable transfer card vs `bridge_names` (objective token-overlap rule + random control)
  - Verify-stage pruning rate (distractors pruned per query)
  - Method/object coherence audit log quality

## New Conventions Proposed

**C-38 (verify-stage blindness)**: The verify prompt is authored by a subagent with NO access to the benchmark, `cross_bridges_ground_truth.json`, `bridge_names`, or any pair/corpus file. It is hashed and committed BEFORE any verification runs.

**C-39 (verify-stage inputs)**: Per candidate, the verify stage sees ONLY {query_title, query_abstract, hyp_target_field, hyp_generic_object, hyp_abstract, candidate_title, candidate_abstract} — no paper IDs, no benchmark context, no other candidates.

**C-40 (pruning rule)**: Candidates with `method_coherence = false` are demoted to rank ∞ (removed from top-k). Ties in the remaining candidates are broken by candidate arxiv_id lexicographic (C-19).

## Open Risks

1. **Verify-stage leakage**: The verify prompt might inadvertently encode benchmark knowledge (e.g., "epidemiology" examples that hint at percolation bridges). Mitigation: blind authoring + hash-before-use + random-control check.

2. **Cost**: The verify stage adds O(#queries × #candidates) LLM calls. For the MVP corpora (36/35 candidates), this is manageable (~1260 calls), but scales linearly with corpus size. Mitigation: cheap gate first, then optimize (batch calls, CAS fallback).

3. **Honest ceiling = TIE again**: The cascade might tie the LLM at 0.60 (recovering pair03/04/05 but missing pair01/06, like everyone else). This is still a win (bank the artifact + cheap retriever), but not a strict beat. Mitigation: pre-register the TIE as the expected outcome and plan the PIVOT (verify-stage tuning) accordingly.

4. **Over-pruning**: The verify stage might prune true positives (e.g., pair01's diffuse review side_b). Mitigation: audit the pruned-card log for false negatives; include a "conservative" ablation (prune only if method_coherence AND object_difference are both false).

5. **Under-pruning**: The verify stage might not prune enough distractors to lift recall@10 over HyDE-alone. Mitigation: include an "aggressive" ablation (prune if method_coherence OR object_difference is false).

## SUMMARY FOR REVIEW

### Key Design Decisions

1. **Reuse EXP-RS-19's proven recall stage**: The HyDE generation mechanism is frozen (prompt, scorer, generations) — we're adding verification, not re-tuning generation.

2. **Blind verify stage**: The verify prompt is authored without benchmark access and hashed before use, mirroring C-31's blindness discipline.

3. **Cheap forward gate first**: Run the full cascade on the 5 Feynman side_a queries (5 × 36 candidates = 180 verify calls) before committing to the modern corpus or ablations. This catches early failures cheaply.

4. **Pruning via method coherence**: The verify stage demotes candidates that fail the method-coherence check, effectively removing them from the top-k. This directly targets the distractor inflation that killed EXP-RS-19.

5. **Honest TIE as expected outcome**: Pre-register that tying the LLM at 0.60 is the modal outcome (nobody recovers pair01/06) and plan the PIVOT (verify-stage tuning) accordingly.

### GATE/KILL Thresholds

- **Cheap gate (P1)**: Feynman forward recall@10 ≥ 3/5 AND pair04 in top-10. If this fails, KILL before any modern/reverse/ablation spend. This is the same gate as EXP-RS-19, but now the cascade must pass it.

- **ADVANCE**: Strict double-beat (≥0.80 Feynman AND > frozen modern bar). This is ambitious but decisive.

- **PIVOT (expected)**: TIE at 0.60 Feynman AND ≥ frozen modern bar AND P4/P5 hold. This banks the artifact pipeline and justifies verify-stage tuning.

- **KILL**: If the cheap gate fails, the cascade has no signal → escalate out of lexical intermediates.

### Open Questions for Human/Claude

1. **Verify-stage implementation**: Should the verify stage use LLM (Claude) or CAS (computational audit via keyword matching)? LLM is more flexible but costlier; CAS is cheaper but might miss nuanced coherence checks.

2. **Prompt design**: The verify prompt needs to elicit method/object coherence checks without leaking benchmark knowledge. Should we use a structured output (JSON with binary flags) or free-text rationale?

3. **Pruning rule**: Should we prune on `method_coherence = false` alone, or require both `method_coherence = false` AND `object_difference = false`? The latter is more conservative but might under-prune.

4. **Ablations**: Should we include aggressive/conservative pruning ablations, or keep the headline arm simple (prune on method_coherence alone)?

5. **Modern corpus**: Should we run the modern baseline now (before the cascade) or only if the Feynman gate passes? Running it now closes the garden-of-forking-paths hole but costs extra LLM calls.
