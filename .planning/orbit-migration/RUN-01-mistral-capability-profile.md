# Orbiter Migration — Run 01: Mistral/Devstral capability profile & the supervision-reduction loop

*research-synergy · Supervised tier · 2026-07-14 · first supervised pi/orbiter run.*
*Purpose: document what Mistral struggles with, how to aid it, and the repeatable
Mistral-self-debrief → Claude-augment loop that lets pi sessions need less supervision.*

---

## 1. The run

| | |
|---|---|
| **Task** | Draft the pre-registration for EXP-RS-20 (generate→verify cascade #3), the thread's own recommended next experiment after HyDE. |
| **Harness / model** | `pi` (`@earendil-works/pi-coding-agent@0.80.3`), headless `-p`, **mistral/devstral-latest** (genuine non-Anthropic independence test). |
| **Guardrails** | Tools `read,write,edit` (no bash); draft-only; git-clean checkpoint `f31e379`; brief forbids editing THREAD.md / CONVENTIONS.md / the vault, and forbids running any experiment. |
| **Output** | `.planning/research/DRAFT-EXP-RS-20-prereg.md` (Mistral, unedited record) + `.claude-revised` version + `.mistral-debrief.md`. |
| **Claude grade** | **Sensible, integrity-clean, ~85–90% of a Claude first draft.** All cited numbers grounded (no fabrication); honored pre-registration conventions; proposed C-38/39/40 without editing locked C-31..C-37; blast radius respected exactly. Needed a 4-fix revision to reach LOCK-ready. **Not significantly worse than Claude.** |

---

## 2. Mistral/Devstral capability profile (empirical + vault-corroborated)

Method: Mistral self-debriefed its own draft; Claude then diffed that self-assessment against its
independent cross-check. The gap between the two IS the capability signal.

**Strength — local under-specification awareness (high recall).** Mistral reliably flags every place it
*locally* invented a choice or left a gap: it named the invented pruning rule, the under-specified
verify I/O contract, the undecided modern-baseline timing, the `...` hash placeholders, even a subtle
convention-carryover risk (C-33 scoring) Claude hadn't flagged. Its "where I guessed" list is an
accurate map of its own gaps. Self-scored 7/10 — well-calibrated.

**Weakness — synthesis-class blind spots (systematic).** The four fixes Claude had to add were *all*
invisible to Mistral's self-debrief, and they cluster into one class:

| Fix Claude added | Mistral's self-debrief | Blind-spot class |
|---|---|---|
| KILL gate too strict (would kill a real 0.20→0.40 gain) | **Inverted it** — worried the PIVOT was "too lenient, raise to 0.65"; opposite direction | **Directional judgment** — knows a threshold is uncertain, mis-judges which way |
| "no signal" KILL rationale contradicts the proven K=1 tie | **Missed** — reproduced C-37 wording; didn't notice the thread flagged the contradiction | **Global consistency** — local choice vs a fact established elsewhere in the corpus |
| Undefined 0.60–0.80 decision band | **Partial** — saw a threshold-*value* question, missed the *coverage gap* | **Structural completeness / exhaustiveness** |
| Reverse-direction ablation | **Absent entirely** | **Unprompted scope** — surfacing what the brief didn't cue |

**Vault corroboration (not a new discovery — a rediscovery).** This maps precisely onto the vault's
already-ratified **synthesis exception** in `[[model-effort-routing]]`: *"pre-calibration froze a
fixture showing both local and Mistral tiers score 0–2/5 on multi-link synthesis … T2-synthesis and all
of T3 cross the cascade to anthropic."* All four blind spots are multi-link-synthesis errors (each
requires holding the whole design against facts scattered across THREAD.md / CONVENTIONS / the verdict).
The empirical run independently reproduced the fixture finding on a live task. **The supervisor should
keep exactly the synthesis load; Mistral keeps the drafting + local self-audit load.**

---

## 3. The augmented debrief (Mistral self-report + the blind spots it missed)

**Mistral filed (Part A/B/C, accurate):** hardest = translating the abstract cascade into concrete
conventions; guessed on = pruning rule, verify I/O, convention carryover, gate reuse; least confident =
pruning rule, new conventions, PIVOT threshold; asked for = "one sentence specifying the verify stage's
inputs/outputs" in the brief. Self-critique: vague pruning rule, under-specified verify outputs, missing
modern-baseline-timing decision. **All valid, all local.**

**Claude adds (the synthesis-class blind spots Mistral could not self-see):**
1. Its PIVOT-threshold worry had the **direction backwards** — the real defect was a KILL gate set at the
   0.60 bar, which would kill a genuine improvement. (Aid: pre-anchor thresholds with reference points.)
2. Its KILL rationale **contradicted a proven thread fact** (signal survives; K=1 ties 0.60). It cannot
   catch this by local review — it requires cross-referencing the verdict. (Aid: consistency checklist.)
3. Its decision bands were **non-exhaustive** (0.70 falls in a gap). It framed this as a value question,
   not a coverage question. (Aid: completeness checklist — "bands must tile the line.")
4. **Reverse direction never occurred to it** because nothing in the brief cued it. (Aid: pre-seed the
   standard ablation menu in the brief.)

---

## 4. Aid design — how to make Mistral need less supervision

Each aid targets a weakness class and pre-empts a supervision pass. Homes per the vault consult
(extend existing pages — do NOT create a new standalone profile page):

| Weakness class | Aid | Vault home |
|---|---|---|
| Directional judgment | **Pre-anchored numeric reference table** in the brief (e.g. "recall stage enters at 0.20; null=0.40; bar=0.60; the KILL gate must sit BELOW the null, not at the bar"). Mistral then places thresholds instead of judging direction. | brief scaffolding → `[[prompting-playbook]]` Mistral section |
| Global consistency | **A "facts-to-honor" checklist** in the brief, quoting the load-bearing established results ("signal survives — do NOT write 'no signal'"). Later: reflex cards from promoted `wiki/concepts/*`. | `[[prompting-playbook]]` + eventually `[[shell-safety-reflexes]]`-style cards |
| Structural completeness | **A required-coverage checklist** ("decision bands must be exhaustive; cover forward AND reverse"). | brief template |
| Unprompted scope | **Pre-seeded ablation/consideration menu** in the brief so standard moves aren't left to recall. | brief template |
| Local under-specification (its strength — lean on it) | **Pre-decide the invented choices** in the brief (Mistral explicitly asked for the verify I/O contract). Its honest "where I guessed" list then becomes the precise double-check map. | brief template |

**Concrete artifact:** a reusable **pi research-drafting brief template** carrying (a) the anchored
reference table, (b) the facts-to-honor block, (c) the coverage checklist, (d) the pre-seeded ablation
menu, (e) explicit pre-decided contracts for anything the model would otherwise invent. This is the
single highest-leverage supervision-reducer — it moves the synthesis load from *post-hoc Claude review*
to *pre-loaded brief*, exactly where Mistral is strong (following an explicit scaffold).

---

## 5. The repeatable loop: Mistral self-debrief → Claude cross-family augment

```
pi/Mistral drafts (T1/T2 task)
   → pi/Mistral runs scribe-debrief  [ratified: model-effort-routing pins /scribe-debrief T1→Devstral]
        → files Part A/B as `Verified: tentative`   [scribe-debrief's normal contract]
   → Claude cross-family augment pass                [fills synthesis-class blind spots Mistral cannot self-see]
        → the delta becomes: agent-weaknesses W-row (Model=Devstral) + briefing-feedback preload row
   → /wiki-caretaker upgrades tentative → verified    [existing sweep]
   → next pi brief auto-carries the aids              [briefing-feedback → CLAUDE.md/AGENTS.md/registry preload lane]
```

**Why this fits the existing model (consult-grounded):**
- A **Mistral-run self-debrief is not a deviation — it IS the ratified model** (`[[model-effort-routing]]`
  pins `/scribe-debrief` to T1→Devstral; it's a bounded transcript scan, below the synthesis-exception
  threshold, so it doesn't self-escalate).
- **Claude augmenting is the honest cross-family review.** The vault's constitution states *"no
  LLM-as-verification-oracle within one model family — parallax exists precisely to fix this."* Parallax
  (P3.1) is **not yet built**; until it ships, **Claude IS the interim heterogeneous verifier**, and the
  ratified **synthesis-exception precedent** (Mistral 0–2/5 → escalate to Anthropic) is the citation that
  makes "Claude reviews Mistral's debrief" a principled rule, not an ad-hoc habit.
- The whole loop rides existing feed-forward plumbing: `briefing-feedback` (scribe-debrief Q6) →
  `/gardener`/`/wiki-caretaker` → a CLAUDE.md/AGENTS.md preload line or registry `Key Conventions` entry.

---

## 6. Proposed vault edits (ready to ratify — NOT yet applied; Supervised-tier originations)

1. **`[[prompting-playbook]]` Mistral section** — append the aid pattern: *"For synthesis-class drafting
   (pre-registrations, plans, anything cross-referencing scattered facts): pre-anchor thresholds with a
   reference table, supply a facts-to-honor block, require exhaustive-coverage + a pre-seeded ablation
   menu. Mistral is strong at following an explicit scaffold and at self-reporting local gaps; it is weak
   at directional judgment, global consistency, and completeness — pre-load those, don't expect them."*
2. **`[[agent-weaknesses]]`** — add a **`Model`** column to the Observations table (extend, don't fork);
   file W-row: *Devstral · synthesis-class blind spots (directional/consistency/completeness/unprompted-
   scope) invisible to its own self-debrief · reproduces model-effort-routing synthesis exception on a
   live task · mitigation = pre-loaded brief scaffolding + Claude cross-family augment.*
3. **`briefing-feedback.md`** — a `gap | proposed briefing change` row: *pi research-drafting briefs must
   carry the anchored reference table + facts-to-honor + coverage checklist + ablation menu.*
4. **Staleness flag** (already queued by the consult): promote *"no LLM-as-verification-oracle within one
   model family"* into `[[do-not-build]]` as a numbered gate once Parallax ships (currently only stated in
   an analysis page).

---

## 7. DONE: the mechanized cross-family panel (interim Parallax) — and what it caught

The manual Claude-augment step of §5 was mechanized as a real `orbiter-run` fanout: a
**3-family heterogeneous audit panel** (local Devstral via llama.cpp + Mistral-small + Claude-sonnet-4-5)
over the revised EXP-RS-20, each judge tool-less and independent, parallax owning the honesty rule.
Workflow: `orbiter/examples/audit-prereg.orbiter.ts`. Run `audit-rs20-v2`: `success · 6 agents · 14.7 ergs`.

**Real orbiter bug found + fixed (the reason this oracle was never verified).** First run: `error ·
3 agents · 0.0 ergs`, no journal. Root cause (via a direct `makeRealAgentFn` probe): standalone
`orbiter-run` can't resolve the pi SDK — `@earendil-works/pi-coding-agent` is a **peerDependency**
present only as a *global* install, so `adapter.ts` threw `ERR_MODULE_NOT_FOUND` on every spawn. The
engine had only ever run as a pi *extension* (where pi supplies the SDK), never headless. Fix: symlink
the global SDK into the orbiter package's `node_modules` (+ a `.gitignore`). A real Mistral call then
returned proper usage. **Proposed durable fix:** document the link step in the orbiter README's
standalone-run section (or add a `postinstall`/`npm link` note).

**What the panel caught — in Claude's own revision (the supervisor's blind spots):** both successful
judges (local Devstral + Mistral) converged `overall: minor` and *independently* flagged that the
**`≈ 0.60` threshold was not operationally measurable** and overlapped its neighbours. Mistral
additionally caught that the **WEAK/SOFT decision bands lacked a modern-corpus condition** — a real
coverage hole letting a "bank the artifact" decision through on good Feynman but regressed modern recall.
Both are legitimate; both are now fixed in `DRAFT-EXP-RS-20-prereg.REVISED.md` (tagged `PANEL REVISION`):
crisp half-open bands replacing `≈ 0.60`, and the modern gate on every proceed band. **The mechanized
cross-family panel found defects the human-supervising Claude had missed — direct evidence the loop
reduces, not just relocates, supervision.**

**Panel-robustness findings (for the workflow, not the draft):**
- The **Claude-sonnet judge `schema_fail`ed** (invalid JSON even after retry) while the two
  smaller-model judges passed — the strict zero-dep `shape()` validator is brittle against a model that
  wraps output in prose/thinking. Fix: loosen extraction (accept the largest JSON object substring) or
  fall back to free-text capture so a schema miss doesn't drop a whole family.
- **Local Devstral over-flags** (called P1/P4 "not measurable" when the doc defines them) — lower
  precision than Mistral, consistent with §2's profile. Usable as a panelist; not as a sole judge.

**Next candidates:** LOCK EXP-RS-20 (human go/kill/pivot); wire the panel behind a `gate()` so a flagged
audit pauses for ratification; harden the `schema_fail` path; then run the panel as a standing pre-LOCK
check on future pre-registrations.
