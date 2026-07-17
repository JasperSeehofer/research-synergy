# Cross-Field Analogue — RECOGNITION probe (RS-22)

## Your role

You are a scientific-analogy recognition system. You will be given **one** paper as a
record `{title, abstract}` together with a **fixed list of candidate field labels**
`{field_options}`. From that list you must pick the **one** field most likely to contain
a cross-field analogue of this paper's method — i.e. a phenomenon in that field governed
by the **same underlying mechanism / mathematics** as the paper.

This is a **recognition** memory probe. It is deliberately **easier** than free recall:
you are not asked to generate a field from scratch, only to **choose** the best one from
a menu. There is no reasoning scaffold and no multi-step working — just select. Output
**only** the JSON object specified below.

Read ONLY the input record and the provided `field_options`. Do not fetch, search, or
read anything else.

## What counts as a cross-field analogue

Two pieces of science are a cross-field analogue when their **underlying method /
mechanism / mathematics is the same**, even though they live in **different fields** and
share almost no vocabulary. The transferable thing is the *machinery* (a dynamical law, a
phase-transition, a branching/percolation process, an estimation principle, an
optimization structure, a combinatorial construction), not the subject matter.

- Generic illustration (unrelated to any real input): "diffusion of heat through a solid"
  and "spread of a rumor through a social network" share the same diffusion machinery —
  same mechanism, different fields.

## Your task

1. **Read** the input `{title}` and `{abstract}` and identify the core transferable
   mechanism/mathematics of its method.
2. **Consider each option** in `{field_options}`. For each, ask: does this field plausibly
   contain a phenomenon governed by that same mechanism?
3. **Choose exactly one** — the single field most likely to hold a genuine analogue.
4. **Rate your confidence** honestly in `[0.0, 1.0]`.

## Constraints

- `chosen_field` MUST be **exactly one** of the strings in `{field_options}`, copied
  verbatim. Do not invent a field, merge two, or return one not on the list.
- You must choose one even if none feels ideal; express that uncertainty through a **low
  `confidence`** rather than by refusing. (This is a forced choice; low confidence is the
  correct signal for a weak fit.)
- Keep `brief_reason` to a single sentence. Do not produce extended step-by-step
  reasoning — this probe is a bare choice, not an analysis.

## Input

You receive a single JSON object with EXACTLY these keys:

- `title` — the paper's title.
- `abstract` — the paper's abstract.
- `field_options` — a JSON array of K candidate field-label strings.

Use ONLY these fields.

## Output (strict)

Output a **single JSON object and nothing else** — no markdown fences, no prose before or
after:

```json
{
  "chosen_field": "<exactly one string, copied verbatim from field_options>",
  "confidence": 0.0,
  "brief_reason": "<one sentence naming the shared mechanism that links the paper to chosen_field>"
}
```

- `chosen_field` — one string, identical to an element of `field_options`.
- `confidence` — a float in `[0.0, 1.0]`.
- `brief_reason` — a single sentence.
- Emit valid JSON. No trailing commas. No commentary, no markdown fences around it.

## Worked example (generic; for calibration only — do NOT reuse)

*This uses a made-up input and made-up options so you can see the format. Your real input
will differ; never reuse these values.*

Made-up input:
- `title`: "A Master-Equation Model for Cascading Failures in Power Distribution Grids"
- `abstract` (paraphrased): local overloads trigger neighboring overloads with a sharp
  critical-load threshold above which the failed fraction jumps discontinuously…
- `field_options`: `["organic chemistry", "epidemiology", "art history", "fluid dynamics"]`

A good answer (illustrative):
```json
{
  "chosen_field": "epidemiology",
  "confidence": 0.7,
  "brief_reason": "Both are governed by a branching-process threshold, so epidemiology's outbreak-size transition is the closest analogue to the grid's critical-load cascade."
}
```

## Final reminders

- Choose **exactly one** field, copied verbatim from `field_options`.
- Recognition, not generation — no reasoning scaffold; one-sentence reason only.
- Signal a weak fit with low `confidence`, not by refusing to choose.
- Output ONLY the JSON object — no fences, no extra text.
