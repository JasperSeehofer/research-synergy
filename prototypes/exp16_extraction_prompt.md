# EXP-RS-16 blind schema extraction — instructions

You extract a **role-typed relational schema** for ONE scientific paper. Work only from the
`{idx, arxiv_id, title, abstract}` record at the input path you are given. **Do NOT** read any other
file, do NOT use web search or any other tool except Read (the one input file) and Write (the one
output file). Do NOT try to guess which other paper this might be analogous to — just faithfully
encode THIS paper's core mechanism. You will not see any other paper; that is intentional (blind
extraction).

## What to produce

A JSON object `{"entities": [...], "relations": [...]}` capturing the paper's core **mechanism**,
with ALL domain-specific nouns alpha-renamed to opaque placeholders (X1, X2, …) so the schema is
domain-blind.

### Entities
Each entity: `{"id": "e1", "role": <closed vocab>, "placeholder": "X1", "gloss": <string>}`
- `role` MUST be exactly one of (else use `"other"`):
  `control-parameter, order-parameter, coupling, conserved-quantity, threshold, state-variable,
   rate, interaction-network, external-field, noise-source, observable, other`
- `placeholder` is opaque (X1, X2, …).
- `gloss` describes the entity's **role/function generically** (e.g. "a scalar tuning parameter",
  "a collective aggregate that becomes non-zero past a critical point") and MUST NOT reveal the
  specific domain noun (no "temperature", "opinion", "infection", "price", "species", etc.).

### Relations
Each relation: `{"src": "e1", "dst": "e2", "type": <closed vocab>}`
- `type` MUST be exactly one of:
  `CAUSES, UNDERGOES-TRANSITION-AT, CONSERVED-UNDER, COUPLES, INCREASES-WITH, DECREASES-WITH,
   DEPENDS-ON`
- Prefer **deep, interconnected** relations (the mechanism), not a flat list of attributes.
- Every `src`/`dst` must reference a declared entity `id`.

### Sizing
Typically 4–10 entities and 3–10 relations. Encode what the abstract actually supports; do not invent.

## Output
Write ONLY the JSON object to the output path you are given (create the parent dir if needed).
Then reply with exactly one line: `NN done: <n_entities>e/<n_relations>r`.
