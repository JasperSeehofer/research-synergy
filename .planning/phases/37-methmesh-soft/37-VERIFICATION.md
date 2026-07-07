# Phase 37 — EXP-RS-18 verification: MethMeSH-Soft (archetype-similarity scoring)

**Verdict (2026-07-07): KILL the mechanism-archetype LBD line.** The pre-registered soft
tagging-recall gate (C-29) FIRED: even with a blind, frozen archetype-adjacency graph, the Feynman
soft tagging-recall reaches only **2/5 < 3/5**. P1 FALSIFIED → per the locked gate, KILL. Both the
exact-match (EXP-RS-17) and the similarity/adjacency (EXP-RS-18) forms of mechanism-ontology tagging
fail the Feynman benchmark bar. Fall back to slot-frames (#2, human-pre-approved).

## What was pre-registered (LOCKED — THREAD.md, CONVENTIONS C-27..C-30)

Keep the frozen vocab (C-22), the 22 blind endpoint tags (C-23), and both corpora; change ONLY the
matching rule: exact-ID overlap → soft overlap over a **blind, frozen archetype-adjacency graph**
(C-27). Soft neighbor-expanded signature, decay s=0.5, score = soft-sig cosine − λ·lexical (C-28).
Cheap soft gate (C-29) run FIRST on the existing 22 tags. Over-abstraction guard (C-30).

LOCKED predictions: **P1** soft gate PASSES (Feynman ≥3/5, up from exact 1/5; modern ≥4/6). **P2**
soft recall@10 > exact-match on Feynman ∧ > 0.15 floor. **P3** soft recall@10 ≥ modern baseline ∧
competitive with 0.60. **P4** guard holds ∧ soft > lexical ∧ soft(s=.5) ≥ exact(s=0). **P5** matched
adjacent archetype maps to `bridge_names`. **GATE:** ADVANCE iff P1∧P3∧P4∧P5 · PIVOT iff P1∧P2∧P4
ties baseline · **KILL the mechanism-archetype line iff P1 fails** OR P4 fails → fall back to #2.

## What was executed

1. **Blind adjacency (C-27).** A blind subagent (no benchmark access) built
   `data/archetype_adjacency.json` from the 125 vocab glosses only: 124 nodes, 427 directed edges
   (out-degree ≤6), symmetric mean-degree 3.84, 1 isolated node. Committed UNMODIFIED (the subagent's
   exact output); SHA-256 `620d9c0f25662da0e1ec2c6fd9e16b646b71ef4c07f6fff13debefa4b6f61604`. The
   gate was computed read-only on this frozen artifact (no post-hoc tuning).
2. **Soft gate (C-29)** computed on the 22 existing endpoint tags, exact-OR-adjacent matching.
   Distractors NOT tagged (gate short-circuited, by design).

## Result — the soft gate (decisive)

| corpus | exact (EXP-RS-17) | **soft (EXP-RS-18)** | gate | verdict |
|---|---|---|---|---|
| **Feynman** | 1/5 | **2/5** | ≥3/5 | **FAIL** |
| Modern | 4/6 | **5/6** | ≥4/6 | PASS |

Adjacency genuinely helped — Feynman 1→2/5 (pair01 now links `hub-dominated-spreading ~
simple-contagion-spread`; pair03 still links), modern 4→5/6 (m03 now links `random-matrix-spectral ~
spectral-eigenmode`) — but **not enough to clear the Feynman gate**. **P1 = (Feynman ≥3/5 AND modern
≥4/6) → FALSIFIED.**

Why the three Feynman pairs still fail (exact-or-adjacent, on the frozen blind graph):
- **pair04 percolation↔epidemics:** side_a tagged `{giant-connected-component, birth-death-branching}`,
  side_b `{compartmental-flow-model, simple-contagion-spread}`. The blind gloss-adjacency did NOT link
  `giant-connected-component` to `compartmental-flow`/`simple-contagion` — the textbook
  percolation≈epidemic-threshold equivalence is a *domain-knowledge* link, not visible from the
  mechanism glosses alone. (Compounded: the percolation paper wasn't even tagged `percolation-threshold`.)
- **pair05 Lotka-Volterra↔markets, pair06 Turing↔economy:** the *economics/markets* side was tagged
  with generic archetypes (`mass-action-bilinear`, `scale-invariance`) that are neither identical nor
  adjacent to the source side's specific mechanism (`turing-instability`, `competitive-exclusion`).

## Verdict vs each locked prediction

- **P1 — FALSIFIED** (Feynman soft 2/5 < 3/5). KILL condition met.
- **P2, P3, P4, P5 — NOT EVALUATED** (soft gate short-circuited the full eval, by design). P2 is
  partially observable: soft *did* lift Feynman tagging-recall over exact (2 vs 1), but not the
  retrieval bar.

## Interpretation — two clean experiments refute the mechanism-archetype line

Exact-ID (EXP-RS-17) and adjacency-soft (EXP-RS-18) both fail the Feynman gate. The failure is now
understood at two levels: (1) **exact-match brittleness** (EXP-RS-17) — solved partially by adjacency;
(2) a **residual representational gap** the adjacency can't close — the cross-domain analogies that
remain (percolation≈epidemic, reaction-diffusion≈spatial-economy) require *domain-knowledge*
equivalences that are invisible from field-agnostic mechanism glosses, AND the blind tagger applies
*generic* archetypes to the non-physics (economics/markets) side. A frozen field-agnostic mechanism
ontology — however matched — cannot represent these bridges on this benchmark. The modern held-out
set fares better (5/6) precisely because its pairs share more *identical* named mechanisms
(Kuramoto↔Kuramoto, SIR↔SIR), i.e. they are less "cross-domain" in the archetype sense.

This is a clean, mechanistic method-negative for the whole mechanism-ontology generator family (#4),
consistent with the brainstorm's "fixed-vocabulary granularity bind" risk. It does NOT refute the
broader semantic-conceptual hypothesis — the full-context LLM baseline still recovers these at 0.60.

## Constructive residue

- Frozen blind vocab (125 archetypes) + blind adjacency graph (427 edges) + 22 blind tags — a
  reusable, auditable mechanism-ontology artifact set.
- A two-level diagnosis of why fixed mechanism ontologies fail cross-domain LBD (brittleness +
  domain-knowledge-equivalence gap + generic-tagging of non-physics sides).
- The reusable harness (tagging, scoring, adjacency, gate) and the leakage-controlled modern corpus.

## Forward path

Per the locked EXP-RS-18 gate, **KILL the mechanism-ontology line (#4, both forms) → fall back to
slot-frames (#2)**, the human-pre-approved next generator (asymmetric problem↔method typed transfer;
brainstorm §2 #2). The go decision to start #2 (a fresh build + pre-registration) is the human's.
