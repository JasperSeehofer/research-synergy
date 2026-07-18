# EXP-RS-23 — Phase 42 Pre-registration: Mechanism-Reduction Substrate

- **Status**: pre-registered; predictions frozen BEFORE running (2026-07-18).
- **Kind**: retrieval-substrate experiment. Direct successor to EXP-RS-21 (dense embeddings, KILLED)
  and a direct test of the human's question: *does an LLM "Feynman reduction" of each paper (structure)
  beat raw full-text comparison, or is no assisting structure needed?*

## Motivation

EXP-RS-21 KILLED dense-embedding retrieval for ONE documented reason (its pre-registered load-bearing
risk, confirmed): **topical/field dominance** — whole-abstract embeddings encode the paper's *field* far
more strongly than its transferable *mechanism*, so same-field distractors sit closer than the true
cross-domain analogue (bge/gte forward recall@10 = 0.20, BELOW the 0.40 lexical null). The analogy signal
survives only under full-context LLM reasoning (0.60 old model / ~1.00 Opus 4.8).

**Hypothesis (H-RS-reduction):** if each paper is first distilled by an LLM to its **field-neutral core
transferable mechanism** (a rich natural-language "Feynman reduction", NOT a lossy schema/tag/closed
vocab) and *then* embedded, the embedding retriever beats its raw-abstract self and the lexical null —
because the reduction strips the field vocabulary that drowned the mechanism signal in RS-21. This is the
one form of "structure" the chapter has not tried: LLM-generated free-text reduction, not a lossy code.

## Design (apples-to-apples with RS-21 — ONE thing changes: the text that gets embedded)

- **Corpus/pairs/pool/metric FROZEN identical to RS-21/EXP-RS-16:** Feynman `mvp_corpus.json` (36 papers),
  5 evaluable pairs, C-19 conditional retrieval (given side_a, rank all 35 others, ties → arxiv_id
  lexicographic), **forward** primary, recall@{1,5,10} + MRR.
- **The reduction** = the FROZEN blind mechanism probe `rs22_probe_mechanism.md` (SHA `72de2252…`), run on
  each paper's `{title, abstract}` ONLY (fresh session, no partner, no benchmark → no pair/answer-key
  leakage; same blindness as RS-22). Output `{core_mechanism, brief_reason}`; the reduction text =
  `core_mechanism`.
- **Encoder FROZEN identical to RS-21 headline:** `bge-large-en-v1.5`, symmetric (no instruction), CPU,
  L2-normalized (reuse `embed_score.encode_st`). The ONLY change vs RS-21 is TEXT: raw `title+'. '+abstract`
  → the `core_mechanism` reduction string.
- **Arms compared (same pool, same metric):**
  - `reduction_bge` — bge over the mechanism reductions (**the test arm**).
  - `raw_bge` — bge over raw abstracts (reproduces RS-21 = 0.20; control that isolates the reduction).
  - `lexical_null` — C-17 TF-IDF over raw abstracts (= 0.40; the memory-free floor to beat).
  - `llm_rawtext` — the brute-force LLM re-rank ceiling (0.60 old / ~1.00 Opus 4.8; context, not a floor).
- **Variant (descriptive):** `reduction_bge_full` = bge over `core_mechanism + '. ' + brief_reason`.

## Predictions (FROZEN before any reduction is generated or scored)

- **P1 (the test — beats the memory-free floor):** `reduction_bge` forward recall@10 **> 0.40** (the
  lexical null). A pass = the right structure rescues cheap retrieval where raw embeddings failed.
- **P2 (isolates the reduction, not the encoder):** `reduction_bge` > `raw_bge` (0.20). The lift comes
  from the reduction, since the encoder is identical.
- **P3 (honest ceiling):** `reduction_bge` ≤ `llm_rawtext`. A compressed substrate is not expected to beat
  full-text LLM reasoning (consistent with every prior chapter result); the value of a pass is
  *scalability* (O(N) reductions + free cosine), not beating the ceiling.

## GATE (decided by the frozen predictions; no post-hoc adjustment)

```
ADVANCE  ⇔ P1 ∧ P2   → the field-neutral reduction is a viable SCALABLE first-stage retriever that
                        beats the null RS-21 could not. Next = build the reduction→LLM re-rank cascade
                        (cheap O(N) shortlist → LLM precision), and re-run on the 420-pair benchmark.
PARTIAL  ⇔ P2 ∧ ¬P1  → the reduction concentrates mechanism signal (beats raw embeddings) but still
                        ties/below the lexical null → descriptive; reduction alone insufficient.
KILL     ⇔ ¬P2       → even an LLM field-neutral reduction does not beat raw-abstract embedding →
                        structure cannot be compressed even by an LLM; raw-text LLM reasoning is
                        genuinely necessary → the chapter's negative result deepens (NO compressed
                        substrate — lexical, embedding, or LLM-reduction — recovers the analogy;
                        only full-context reasoning does).
```

Honest expectation: given the whole chapter's pattern (every compression has lost to raw text), the modal
outcome is **PARTIAL or KILL**; an ADVANCE would be the first compressed substrate to beat the null and
is the high-value surprise worth testing cheaply. Confirmatory extension to the 420-pair mined benchmark
only if Feynman ADVANCEs.
