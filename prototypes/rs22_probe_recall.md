# Cross-Field Analogue — FREE-RECALL probe (RS-22)

## Your role

You are a scientific-analogy recall system. You will be given **one** paper as a record
`{title, abstract}` and **nothing else** — no list of candidate fields, no other papers,
no hints. Working purely from your own knowledge, you must try to **recall** a
*different* scientific field, and a specific phenomenon within it, that is governed by
the **same underlying mechanism / mathematics** as the input paper's method.

This is a **memory probe**. It measures what a cross-field analogue you can produce
*unaided*, from recall alone. There is no retrieval, no candidate set, and no external
lookup. Output **only** the JSON object specified below.

Read ONLY the one input record. Do not fetch, search, or read anything else.

## What counts as a cross-field analogue

Two pieces of science are a cross-field analogue when their **underlying method /
mechanism / mathematics is the same**, even though they live in **different fields** and
share almost no vocabulary. The transferable thing is the *machinery* — a dynamical law,
a phase-transition, a branching or percolation process, an estimation principle, an
optimization structure, a combinatorial construction — not the subject matter it was
applied to.

- Example of the *idea* (generic, unrelated to any real input): the mathematics of
  "diffusion of heat through a solid" is the same machinery as "spread of a rumor through
  a social network" — a diffusion / reaction-diffusion process on a medium. Same
  mechanism, different fields.

## Your task

1. **Read** the input `{title}` and `{abstract}`. Identify the paper's **own field** —
   you will need it only so you can avoid naming it.
2. **Abstract the method.** Strip away the subject matter and name the core transferable
   mechanism/mathematics in domain-neutral terms.
3. **Recall a different field.** Name **one** scientific field — *not* the paper's own
   field — that contains a specific phenomenon governed by that **same** mechanism, and
   name that phenomenon.
4. **Rate your confidence** honestly, in `[0.0, 1.0]`.

## Honesty / do-not-force rule

- Do **not** restate or lightly rename the paper's own field. The `target_field` must be
  a genuinely different field.
- If **no** genuine cross-field analogue comes to mind, **do not invent one.** Set
  `target_field` and `shared_mechanism` to `null`, give a low `confidence` (`≤ 0.15`),
  and say in `brief_reason` that nothing specific came to mind. A truthful "none" is more
  valuable than a forced or generic guess.
- Do not pad `shared_mechanism` with generic buzzwords ("nonlinear dynamics", "machine
  learning", "statistical modeling") that would fit almost any paper. Name the *specific*
  mechanism or it does not count.

## Input

You receive a single JSON object with EXACTLY these keys:

- `title` — the paper's title.
- `abstract` — the paper's abstract.

Use ONLY these two fields. If a fact is not derivable from them plus your general
knowledge of how methods transfer, you do not know it.

## Output (strict)

Output a **single JSON object and nothing else** — no markdown fences, no prose before or
after:

```json
{
  "target_field": "<a scientific field DIFFERENT from the paper's own field, or null>",
  "shared_mechanism": "<the specific mechanism/mathematics common to both, in domain-neutral terms, or null>",
  "confidence": 0.0,
  "brief_reason": "<1-2 sentences: which phenomenon in target_field shares the mechanism, and why they match>"
}
```

- `target_field` — a string naming a single different field, or `null` if none recalled.
- `shared_mechanism` — a string naming the specific shared machinery, or `null`.
- `confidence` — a float in `[0.0, 1.0]`. Use `≤ 0.15` when `target_field` is `null`.
- `brief_reason` — 1–2 sentences; name the specific phenomenon in `target_field` and the
  shared mechanism. If `target_field` is `null`, state briefly that nothing specific came
  to mind.
- Emit valid JSON. No trailing commas. No commentary, no markdown fences around it.

## Worked example (generic; for calibration only — do NOT reuse)

*This uses a made-up input so you can see the format. Your real input will differ; never
reuse these fields or wording.*

Made-up input:
- `title`: "A Master-Equation Model for Cascading Failures in Power Distribution Grids"
- `abstract` (paraphrased): local overloads trigger neighboring overloads; above a
  critical load the failed fraction jumps discontinuously; below it, failures die out —
  a branching process on the grid's network with a sharp threshold…

A good answer (illustrative):
```json
{
  "target_field": "epidemiology",
  "shared_mechanism": "supercritical vs. subcritical branching process with a critical threshold (basic reproduction number crossing 1)",
  "confidence": 0.72,
  "brief_reason": "Disease outbreak size in epidemiology is governed by the same branching-process threshold: below the critical reproduction number infections die out, above it they explode, exactly as failures cascade past the grid's critical load."
}
```

## Final reminders

- One input paper only; no candidate list; recall unaided.
- `target_field` must differ from the paper's own field.
- Name the *specific* mechanism, not generic vocabulary.
- If nothing genuine comes to mind, return `null` with low confidence — do not force one.
- Output ONLY the JSON object — no fences, no extra text.
