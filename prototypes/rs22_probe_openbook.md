# Cross-Field Analogue — OPEN-BOOK reasoning probe (RS-22)

## Your role

You are a cross-field method-mapping analyst. You will be given **two** papers,
**paper A** `{title_a, abstract_a}` and **paper B** `{title_b, abstract_b}`, drawn from
**different fields**. Both abstracts are in front of you — this is **open book**. Your job
is to reason explicitly about whether their **underlying methods / mechanisms /
mathematics are the same**, and to write down the **element-by-element mapping** between
them (what in A plays the role of what in B).

This probe **separates "can't recall" from "can't reason"**: because both papers are
supplied, no recall is needed. If the two genuinely share machinery, a competent reasoner
should be able to state the mapping directly from the texts. Output **only** the JSON
object specified below.

Read ONLY the two provided records. Do not fetch, search, or read anything else. Base
every judgment on the given texts plus general knowledge of how methods work — not on any
outside facts about these specific papers.

## What counts as sharing a method

Papers A and B **share an underlying method** when the *same transferable machinery* — a
dynamical law, a phase-transition, a branching/percolation process, an estimation
principle, an optimization structure, a combinatorial construction, a governing equation —
drives both, even though the objects, vocabulary, and fields differ. Shared *vocabulary*
or a superficially similar-sounding problem is **not** shared machinery.

- Generic illustration (unrelated to any real input): "diffusion of heat through a solid"
  ↔ "spread of a rumor through a social network". Mapping: temperature ↔ fraction who have
  heard the rumor; thermal conductivity ↔ transmission rate; the heat/diffusion equation ↔
  the same diffusion equation on the social graph. `shares_method = true`.

## Your task

1. **Read both abstracts** and name each paper's core mechanism in domain-neutral terms.
2. **Attempt the mapping.** List correspondences: for each salient element of paper A's
   mechanism, identify the element of paper B's mechanism that plays the *same role*
   (variables, parameters, the driving equation/process, the transition or threshold, the
   objects being acted on).
3. **Decide** whether they truly share an underlying method (`shares_method`).
4. **Justify** briefly, grounded in the two texts.

## Judgment guidance

- Set `shares_method = true` only if the *machinery* corresponds — the same governing
  process/structure, such that a role-for-role mapping actually holds. If you cannot fill
  in a coherent mapping, they probably do **not** share a method.
- Set `shares_method = false` if the papers merely share keywords, tackle
  superficially-similar problems by **different** mechanisms, or if the only overlap is
  generic ("both use optimization", "both are nonlinear").
- The `mapping` array should contain the correspondences you actually find. If
  `shares_method` is `false`, include any **partial** correspondences you did identify (or
  an empty array if there are none) — the mapping records your reasoning either way.
- Do not force spurious pairings to make the mapping look complete. A short, honest
  mapping is better than an inflated one.

## Input

You receive a single JSON object with EXACTLY these keys:

- `title_a`, `abstract_a` — paper A's title and abstract.
- `title_b`, `abstract_b` — paper B's title and abstract.

Use ONLY these fields.

## Output (strict)

Output a **single JSON object and nothing else** — no markdown fences, no prose before or
after:

```json
{
  "mapping": [
    {"a_element": "<an element of paper A's mechanism>", "b_element": "<the element of paper B that plays the same role>"}
  ],
  "shares_method": true,
  "brief_justification": "<1-3 sentences naming the shared (or mismatched) machinery, grounded in both abstracts>"
}
```

- `mapping` — an array of `{a_element, b_element}` role-correspondence pairs; may be empty
  if no correspondence holds.
- `shares_method` — boolean.
- `brief_justification` — 1–3 sentences; name the shared machinery (if `true`) or the
  decisive mismatch (if `false`), citing what in the texts drove the call.
- Emit valid JSON. No trailing commas. No commentary, no markdown fences around it.

## Worked example (generic; for calibration only — do NOT reuse)

*Made-up papers so you can see the format. Your real inputs will differ.*

Made-up input:
- `title_a`: "A Master-Equation Model for Cascading Failures in Power Distribution Grids"
- `abstract_a` (paraphrased): overloads propagate to neighbors; above a critical load the
  failed fraction jumps; below it failures die out — a branching process with a threshold.
- `title_b`: "Outbreak Size and the Epidemic Threshold in a Contact-Network SIR Model"
- `abstract_b` (paraphrased): infections spread to contacts; above a critical reproduction
  number the outbreak reaches a finite fraction of the population, below it it fizzles.

A good answer (illustrative):
```json
{
  "mapping": [
    {"a_element": "an overloaded node triggering neighbors", "b_element": "an infected individual infecting contacts"},
    {"a_element": "critical load threshold", "b_element": "critical reproduction number (R0 = 1)"},
    {"a_element": "final failed fraction of the grid", "b_element": "final outbreak size in the population"},
    {"a_element": "branching process on the grid network", "b_element": "branching process on the contact network"}
  ],
  "shares_method": true,
  "brief_justification": "Both are supercritical/subcritical branching processes on a network with a sharp threshold separating a vanishing cascade from one that reaches a finite fraction; the grid's critical load plays exactly the role of the epidemic's reproduction number, so the machinery is the same."
}
```

## Final reminders

- Two papers, both supplied — open book; no recall needed.
- Build the role-for-role mapping from the texts; do not invent pairings.
- `shares_method` = shared MACHINERY, never mere shared keywords.
- Output ONLY the JSON object — no fences, no extra text.
