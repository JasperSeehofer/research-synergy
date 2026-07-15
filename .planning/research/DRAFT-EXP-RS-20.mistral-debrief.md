# Self-Debrief: DRAFT-EXP-RS-20-prereg.md

## Part A — Signals for future runs (self → the harness)

### 1. Hardest part
The hardest part was translating the abstract "generate→verify cascade" recommendation from EXP-RS-19 into concrete, pre-registrable conventions and metrics. Specifically:
- **Verify-stage design**: Deciding what inputs the verify stage should see and what outputs it should produce (binary flags vs free-text, method coherence vs object difference). I had to infer this from the high-level description in EXP-RS-19's conclusion.
- **Pruning rule**: Determining how to integrate the verify stage's output into the ranking (demote to rank ∞ vs other options). This required making a concrete choice where the instructions only said "prune distractors."
- **Ablations**: Deciding whether to include aggressive/conservative pruning variants or keep the headline simple. The instructions didn't specify, so I had to choose.

### 2. Where I guessed
- **Line 32-34**: "The cascade beats the brute-force LLM baseline... AND holds up on the modern held-out set" — I assumed the modern set is the primary leakage-controlled comparison, but EXP-RS-19 treated Feynman as primary. I followed EXP-RS-19's pattern but wasn't 100% sure.
- **Line 50-52**: "FAIL → KILL before any modern/reverse/ablation spend" — I assumed the cheap gate should be identical to EXP-RS-19's (GATE-A ∧ GATE-B), but the instructions didn't explicitly say to reuse it. I could have designed a new gate.
- **Line 70-72**: "Reuse C-14, C-17, C-19, C-20, C-24, C-31..C-37" — I assumed all EXP-RS-19 conventions would carry over, but some (e.g., C-33 scoring) might need adjustment for the cascade. I didn't flag this uncertainty.
- **Line 82-84**: "Blind VERIFY prompt... SHA-256 committed BEFORE any verification" — I assumed the verify prompt should follow the same blindness discipline as the HyDE prompt (C-31), but the instructions didn't specify. I mirrored it to be safe.
- **Line 100-102**: "Pruning: candidates with method_coherence = false are demoted to rank ∞" — I invented this pruning rule. The instructions said "prune distractors" but didn't specify how.
- **Line 110-112**: "C-38..C-40" — I proposed three new conventions, but I wasn't sure if all three were necessary or if I should have proposed more/less.

### 3. What I left unresolved and why
- **SHA-256 placeholders**: Lines 68, 74 — I left `...` for the HyDE prompt and generations SHA-256 hashes because I didn't have access to the actual files and the instructions said not to run any code. I flagged this in the open questions.
- **Verify-stage implementation**: Line 126 — I didn't specify whether the verify stage should use LLM or CAS. I left this as an open question because the instructions didn't specify and it's a meaningful design choice.
- **Prompt design**: Line 127 — I didn't specify whether the verify prompt should use structured output (JSON) or free-text. I left this open because it affects the pruning rule.
- **Modern baseline timing**: Line 130 — I didn't specify whether to run the modern baseline now or later. I left this open because it's a cost/rigor tradeoff.

### 4. Brief gaps
The instructions were clear overall, but one addition would have reduced uncertainty:
- **Verify-stage specification**: A concrete example of what the verify stage should do (inputs, outputs, pruning rule). Even one sentence like "The verify stage takes a query, a hypothetical, and a candidate, and returns a binary method-coherence flag" would have helped.

### 5. Least confident
1. **Pruning rule (Line 100-102)**: I invented "demote to rank ∞" but I'm not sure if this is the right way to prune. It might interact weirdly with the tie-break rule.
2. **New conventions (Line 110-112)**: I proposed C-38..C-40 but I'm not sure if these are the right conventions or if I missed any.
3. **PIVOT threshold (Line 58-60)**: I set the PIVOT at "TIE at 0.60" but I'm not sure if this is too lenient or if I should have required a small beat (e.g., ≥0.65).

## Part B — For the reviewer (self → Claude/human)

### 1. Double-check most carefully
- **Pruning rule**: Is "demote to rank ∞" the right way to prune? Does it interact correctly with the tie-break rule (C-19)?
- **New conventions**: Are C-38..C-40 the right conventions? Are they sufficient? Should there be more/less?
- **PIVOT threshold**: Is "TIE at 0.60" the right bar for PIVOT, or should it be higher (e.g., ≥0.65)?

### 2. Simplified/skipped
- **Verify-stage implementation details**: I didn't specify whether to use LLM or CAS, or how to design the prompt. I left this as open questions.
- **Ablations**: I didn't specify whether to include aggressive/conservative pruning variants. I kept the headline simple but this might be too minimal.
- **Cost analysis**: I mentioned cost as a risk but didn't quantify it (e.g., estimated LLM tokens or $ cost).

## Part C — Self-critique

### Weakness 1: Vague pruning rule
**Problem**: I said "demote to rank ∞" but didn't specify how this interacts with the existing ranking. For example, if two candidates are tied at rank 10 and one is pruned, does the other move up to rank 10 or stay at rank 11?
**Fix**: Specify that pruned candidates are removed from the ranking entirely, and the remaining candidates are re-ranked (so ranks shift up to fill the gap).

### Weakness 2: Under-specified verify-stage outputs
**Problem**: I said the verify stage outputs `{method_coherence: bool, object_difference: bool, rationale: str}` but didn't specify how these fields are used. For example, do we prune if either is false, or only if both are false?
**Fix**: Specify the exact pruning condition (e.g., prune if `method_coherence = false`). Also specify whether `object_difference` is used for pruning or just for auditing.

### Weakness 3: Missing modern-baseline timing decision
**Problem**: I didn't specify whether to run the modern baseline now (before the cascade) or later (if the Feynman gate passes). This affects rigor (running it now closes the garden-of-forking-paths hole) but costs extra LLM calls.
**Fix**: Make a concrete decision (e.g., run it now) and justify it (e.g., "to close the garden-of-forking-paths hole, we run the modern baseline now, even though it costs extra LLM calls").

## SELF-ASSESSED QUALITY: 7/10
The draft is structurally sound and follows the instructions, but it has concrete gaps in the verify-stage design and pruning rule that need resolution before locking.