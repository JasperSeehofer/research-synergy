# EXP-RS-20 — Generate→Verify Cascade: generation-based cross-field retrieval with computational verification (design line #3)

> **Provenance of THIS file.** Claude's revision of `DRAFT-EXP-RS-20-prereg.md` (drafted by a
> pi/Mistral-Devstral session, 2026-07-14). Mistral's original is preserved unedited as the record.
> This revision applies four fixes; each is tagged inline `> **CLAUDE REVISION #n**`. Still a DRAFT —
> predictions LOCK only on the human's go. Summary of the four changes:
> 1. **KILL gate de-conflated** — the cheap gate no longer kills a cascade merely for not hitting 0.60;
>    it kills only if the cascade cannot beat the 0.40 lexical null (Mistral set KILL = "recall ≥ 3/5",
>    which would kill a genuine 0.20→0.40 improvement).
> 2. **Exhaustive decision bands** — the 0.40/0.60/0.80 thresholds are now non-overlapping and cover
>    the whole line (Mistral left the 0.60–0.80 band undefined).
> 3. **KILL rationale corrected** — a cascade KILL means "verification cannot rescue the aggregation,"
>    NOT "no signal" (the thread already PROVED signal survives: K=1 ties 0.60). Mistral wrote "no signal."
> 4. **Reverse-direction ablation added** — EXP-RS-19 measured forward only; the transfer is directional.

- **Status**: pre-registration DRAFT (predictions to be LOCKED before any run)
- **Provenance**: Direct follow-up to EXP-RS-19 (HyDE-Bridge). EXP-RS-19 proved the generation mechanism works (pair04 recovered rank 17→4 via an epidemiology hypothetical matching the real epidemics paper) and tied the LLM baseline at K=1 (recall@10 = 0.60), but the pinned K=5 max-pool headline was KILLED by distractor inflation (monotonic: K1=0.60, K3=0.40, K5=0.20). The signal survives the lexical comparator but is throttled by aggregation noise. The thread's recommended forward — the convergence point of both C-37's escalation branch and the design's PIVOT — is a **generate→VERIFY cascade**: keep HyDE's recall stage (proven to convert latent cross-field equivalences into retrievable tokens) and add a VERIFY stage (an LLM/computational check that audits each proposed transfer for method/object coherence and prunes distractors).

## Hypothesis (H-RS-analogy-cascade)

A cross-field analogy = SAME method, DIFFERENT object; it is recoverable by a two-stage cascade:
1. **GENERATE** (HyDE, frozen): from the query alone, generate hypothetical abstracts re-expressing its method in other fields' native vocab (proven in EXP-RS-19 to recover cross-vocabulary pairs the lexical comparator misses).
2. **VERIFY** (LLM/CAS, new): audit each proposed transfer for method/object coherence and prune distractors, targeting the max-pool inflation that killed EXP-RS-19.

The cascade beats the brute-force LLM baseline (recall@10 = 0.60) on the shared Feynman pairs AND holds up on the modern held-out set where the LLM's pretraining advantage is neutralized.

**Discriminating experiment**: Run the full generate→verify cascade on the Feynman MVP and modern held-out corpora, with the HyDE recall stage frozen (reuse EXP-RS-19's blind prompt + scorer + generations) and a new blind VERIFY stage (benchmark-agnostic prompt, hashed before use). Score vs the 0.60 bar + nulls + the auditable transfer-card check vs `bridge_names`.

## Predictions (to LOCK before any run)

> **CLAUDE REVISION #1 + #3 (P1).** The cheap gate is re-cut so it measures whether *verification adds
> value*, not whether the cascade already wins. HyDE-alone's recall stage enters this cascade at 0.20
> (below the 0.40 lexical null). The verify stage's whole job is to prune distractors and lift that. So
> the honest cheap gate is: **does verify rescue the aggregation past the free lexical null?** If not,
> distractor-pruning has failed and the lexical-intermediate line is exhausted — that is the KILL, and
> it is about *aggregation rescue*, not *absent signal* (signal survival is already proven).

**P1 (cheap forward gate — decisive, run FIRST, 5 Feynman side_a queries only)**:
- Cascade forward recall@10 **> 0.40** (strictly beats the C-17 lexical null — i.e. verification does what max-pool HyDE alone could not), **AND**
- pair04 (percolation→epidemics) recovered into forward top-10 (the EXP-RS-19 GATE-B anchor is not lost), **AND**
- ≥1 pair the C-17 null misses (pair01/04/06) recovered.
- **FAIL → KILL** before any modern/reverse/ablation spend. KILL semantics: *verification cannot rescue the K=5 aggregation past free lexical retrieval* → escalate OUT of lexical intermediates (real semantic-embedding substrate or a pure LLM-judge cascade); do NOT run another TF-IDF variant. (NOT "no signal" — EXP-RS-19 proved signal survives at K=1.)

**P2 (verify beats its own recall stage — the value-of-verification test)**:
- Cascade forward recall@10 **> HyDE-alone K=5 max-pool (0.20)** on Feynman. If verify cannot beat the aggregation it is layered on, the verify stage adds only cost.

**P3 (the bar — HONEST TIE expected, not a beat)**:
- Cascade Feynman recall@10 **≥ 0.60** (tie-or-beat the leaky LLM; the modal outcome is an *exact* tie at 0.60 — nobody, including the LLM, recovers pair01/06), so the recall number is NOT the deliverable — the auditable artifact is (P5).
- Modern recall@10 **≥ the frozen modern bar (C-35) AND ≥ 0.833** (no regression vs the modern null).

**P4 (verify lifts precision, mechanistically)**:
- The verify stage prunes **≥2 distractors per query on average** (auditable via the pruned-card log), and the pruned items are disproportionately same-field distractors (the EXP-RS-19 failure mode).

**P5 (auditable artifact — THE PRIMARY DELIVERABLE)**:
- For **≥3/5 recovered Feynman pairs**, the winning transfer card's {method_core ∪ generic_object} tokens match `bridge_names` under the C-36 objective rule; a random `bridge_name` control does NOT match.
- The verify stage's audit log shows ≥1 method/object coherence judgment per recovered pair.

## GATE — decision bands (exhaustive, non-overlapping)

> **CLAUDE REVISION #2.** Bands now tile the entire recall line with no gaps and no overlaps. `R` =
> cascade Feynman forward recall@10. Every outcome maps to exactly one branch.
>
> **PANEL REVISION (orbiter 3-family audit, 2026-07-14).** Two fixes from the mechanized cross-family
> panel (local Devstral + Mistral converged independently): (a) the `≈ 0.60` TIE threshold was not
> operationally measurable and overlapped its neighbours — replaced with crisp half-open intervals so
> `R` maps to exactly one band; the old TIE + SOFT-BEAT bands merge into one `[0.60, 0.80)` PIVOT band.
> (b) the WEAK/SOFT bands lacked a modern-corpus condition, risking a "bank the artifact" decision on
> good Feynman but *regressed* modern recall — every proceed band now carries the modern (`M`) gate.

| Band | Condition (`R` = Feynman fwd recall@10; `M` = modern recall@10) | Decision |
|---|---|---|
| **KILL** | `R ≤ 0.40` (fails P1) | Lexical-intermediate line exhausted → escalate to embedding substrate or pure LLM-judge cascade. Bank the frozen residue; no more TF-IDF variants. |
| **WEAK / PIVOT-caveat** | `0.40 < R < 0.60` ∧ P2 holds ∧ `M ≥ 0.833` (modern not regressed) | Verify rescues past the null but stays below the incumbent (0.60). Bank the cheap retriever + artifact; log that it underperforms the baseline; next build = verify tuning, NOT a scale-up. If `M < 0.833`, treat as KILL-adjacent (do not bank). |
| **PIVOT (expected, modal — subsumes the exact tie)** | `0.60 ≤ R < 0.80` ∧ `M ≥ C-35 bar` ∧ `M ≥ 0.833` ∧ P4/P5 hold | Bank the artifact pipeline + cheap retriever; next build = verify-stage tuning (LLM vs CAS, prompt ablations). The auditable directed transfer card is the value, not the tied number. A beat in `[0.60, 0.80)` is provisional (single-corpus, leakage-suspect) — it PIVOTs, it does not ADVANCE. |
| **ADVANCE** | `R ≥ 0.80` ∧ `M` strictly `> C-35 bar` (strict double-beat) | Generation+verification is a genuine generator. Formalize through the resyn pipeline (bulk-ingest→analyze→export) for the official number; scale the corpus. |

## Setup

**Reused (frozen residue from EXP-RS-19)**:
- Feynman MVP corpus (C-14) + modern held-out (C-24)
- Blind HyDE prompt (`prototypes/hyde_prompt.md`, SHA-256 `<FILL AT FREEZE>`) + scorer (`hyde_score.py`)
- 5 frozen Feynman side_a generations (`data/hyde_hypotheticals_feynman.json`, SHA-256 `<FILL AT FREEZE>`)
- Verified pair04 transfer card (percolation→epidemiology)
- C-19 retrieval metric, sme_lite scorer, C-17 null (Feynman 0.40, modern 0.833)
- C-20 baseline (Feynman 0.60) + C-35 modern bar (frozen before modern scoring)

> **CLAUDE NOTE.** The `<FILL AT FREEZE>` markers replace Mistral's `...` placeholders. Mistral was
> right NOT to invent hashes; making the fill-step explicit closes the loop. These hashes are computed
> and committed at freeze time, before any verify run — the blind-benchmark discipline (C-31 lineage).

**NEW (verify stage)**:
- **Blind VERIFY prompt** (`prototypes/verify_prompt.md`): benchmark-agnostic, authored by a no-benchmark subagent using non-benchmark examples, SHA-256 committed BEFORE any verification.
- **Inputs (per candidate pair)**: the query paper's {title, abstract}, the winning hypothetical's {target_field, generic_object, abstract}, and the candidate paper's {title, abstract}. No paper IDs, no benchmark context, no sight of other candidates (C-39).
- **Output**: `{method_coherence: bool, object_difference: bool, rationale: str}` — a binary audit of whether the proposed transfer is method-coherent and object-different.
- **Pruning (headline rule, C-40)**: candidates with `method_coherence = false` are demoted to rank ∞ (removed from top-k); remaining ties broken by candidate arxiv_id lexicographic (C-19).
- **Frozen before scoring**: the verify prompt SHA-256 + the first 5 Feynman verify logs are committed before any full-corpus scoring.

**Ablations (pre-registered, run only if P1 passes)**:
- **Pruning severity**: headline (`method_coherence=false` → prune) vs conservative (`method_coherence=false ∧ object_difference=false` → prune) vs aggressive (`method_coherence=false ∨ object_difference=false` → prune).
- **Verify backbone**: LLM (Claude/Mistral) vs CAS (computational keyword/rule audit) — the cost/quality trade named in the open questions.
> **CLAUDE REVISION #4 (reverse-direction ablation).** EXP-RS-19 scored the **forward** direction
> (side_a query → side_b candidate) only. The transfer is *directional* (percolation→epidemiology is not
> symmetric). Add a **reverse-direction** ablation: generate hypotheticals from side_b, retrieve side_a,
> report recall@10 per direction. If the cascade is strongly asymmetric, that is itself a finding about
> where the generative bridge is easy vs hard, and it guards against a forward-only artifact.

## Metric

- **Primary**: recall@{1,5,10} + MRR on Feynman AND modern held-out, vs the Feynman 0.60 bar + 0.40 null and the modern frozen bar + 0.833 null.
- **Co-primary / primary-value**: auditable transfer card vs `bridge_names` (objective C-36 token-overlap rule + random control).
- **Secondary**: verify-stage pruning rate (distractors pruned per query, and same-field share); per-direction recall (forward vs reverse); method/object coherence audit-log completeness.

## New Conventions Proposed (continue from C-37; do NOT renumber locked ones)

- **C-38 (verify-stage blindness)**: The verify prompt is authored by a subagent with NO access to the benchmark, `cross_bridges_ground_truth.json`, `bridge_names`, or any pair/corpus file. It is hashed and committed BEFORE any verification runs. (Extends the C-31 blind-generation lineage to the verify stage.)
- **C-39 (verify-stage input closure)**: Per candidate, the verify stage sees ONLY {query_title, query_abstract, hyp_target_field, hyp_generic_object, hyp_abstract, candidate_title, candidate_abstract} — no paper IDs, no benchmark context, no sight of other candidates (prevents cross-candidate leakage / list-position effects).
- **C-40 (pruning rule)**: Headline pruning demotes `method_coherence = false` candidates to rank ∞; ties broken by arxiv_id lexicographic (inherits C-19). Alternative pruning severities are pre-registered ablations, not headline.

## Open Risks

1. **Verify-stage leakage**: the verify prompt might encode benchmark knowledge (e.g. "epidemiology" hints). Mitigation: blind authoring (C-38) + hash-before-use + random-control check (P5).
2. **Cost**: verify adds O(#queries × #candidates) calls (~180 for the 5-query Feynman cheap gate; ~1260 for full Feynman). Mitigation: cheap gate first (P1); CAS-fallback ablation; batch calls.
3. **Honest ceiling = TIE again**: the cascade may tie at 0.60 (recovers 03/04/05, misses 01/06 like everyone). Still a win (bank artifact + cheap retriever), pre-registered as the modal PIVOT outcome — not a failure.
4. **Over-pruning**: verify may prune true positives (e.g. pair01's diffuse side_b). Mitigation: audit the pruned-card log for false negatives; the conservative ablation.
5. **Under-pruning**: verify may not prune enough to beat HyDE-alone (P2). Mitigation: the aggressive ablation.
6. **Directional asymmetry masking** (from CLAUDE REVISION #4): a forward-only score could hide that the bridge is only recoverable one way. Mitigation: the reverse-direction ablation.
