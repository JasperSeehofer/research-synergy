# EXP-RS-17 blind mechanism-archetype tagging — instructions

You tag ONE scientific paper with mechanism archetypes drawn from a FROZEN, closed vocabulary, for a
literature-based-discovery experiment. Work ONLY from the two input files named in your task; do NOT
read anything else, do NOT use web search, and do NOT try to guess which other paper this one might
be analogous to. Tag THIS paper's own mechanism(s) only — this is intentional (blind tagging).

## Inputs (read exactly these two files, nothing else)
1. **Paper record** — the JSON `{arxiv_id, title, abstract}` at the input path you are given.
2. **Mechanism vocabulary** — `/home/jasper/Repositories/research-synergy/prototypes/data/mechanism_vocab_compact.json`,
   a closed list of archetypes, each `{id, label, gloss}`. You may ONLY use ids from this list.

## Task
Identify the **1–5 archetypes** from the vocabulary that best capture this paper's CORE mechanism(s)
— the structural / mathematical machinery it uses or studies, abstracted from its specific domain
(what is *happening* mechanistically, not what field it is in). For each chosen archetype, cite a
short **verbatim** evidence snippet from the abstract (or title) that justifies it.

## Rules
- `archetype_id` MUST be an exact id string that exists in the vocabulary (e.g. `arch-042`).
- Choose the archetypes that most **specifically** match the paper's actual mechanism. Prefer a
  sharp, specific archetype over a broad one; do NOT tag a generic archetype if a more precise one
  fits. Do NOT pad to 5 — assign only genuinely-present mechanisms (typically 2–4).
- Base every choice purely on this paper's own content. Never pick an archetype to set up an analogy.

## Output
Write ONLY this JSON to the output path you are given (create parent dirs if needed):
```
{"arxiv_id": "<the paper's arxiv_id>",
 "tags": [{"archetype_id": "arch-NNN", "evidence_snippet": "<verbatim phrase from the abstract>"}, ...]}
```
Then reply with exactly one line: `TAG done: <arxiv_id> -> <n> archetypes: <comma-separated ids>`.
