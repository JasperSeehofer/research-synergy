# RS-22 prompt set — index

Five FROZEN, benchmark-agnostic prompt templates for a study that measures whether a
language model recovers **cross-field analogies** (two papers from different fields whose
underlying method/mechanism/mathematics is the same) by **memory** or by **reasoning**.

Every prompt is a reusable template with `{placeholders}`; none contains, references, or
assumes any benchmark, pair, or answer key. Each is SHA-frozen before use.

## The prompts and their role in the study

| File | Probe / role | Input | Core output | Function in the design |
|---|---|---|---|---|
| `rs22_probe_recall.md` | **Free-recall** memory probe | one paper `{title, abstract}` only | `{target_field, shared_mechanism, confidence, brief_reason}` | Defines the **memory-absent baseline**: what analogue the model can produce unaided, with no candidates. This is the memory-bearing output scored by the judge. |
| `rs22_probe_recognition.md` | **Recognition** memory probe (forced choice) | one paper + fixed `{field_options}` | `{chosen_field, confidence, brief_reason}` | Easier memory floor: pick the right field from a menu. Recall vs. recognition together bound the **"memory-absent"** condition. |
| `rs22_probe_familiarity.md` | **Paper-familiarity** control | paper **title only** | `{recognized, stated_result_or_null, confidence}` | Control: does the model specifically *know this paper*? Separates recalling the analogy from merely having memorized the source paper. |
| `rs22_probe_openbook.md` | **Open-book reasoning** control | two papers A and B `{title, abstract}` | `{mapping, shares_method, brief_justification}` | Control: with both papers supplied, can the model *reason* the mapping? Separates **can't-recall** from **can't-reason**. |
| `rs22_judge_semantic.md` | **Blind semantic-equivalence judge** | stated `{target_field, shared_mechanism}` + held-out `{reference_field, reference_mechanism}` | `{field_match, mechanism_match, overall_equivalent, rationale}` | Scores the recall probe's memory-bearing output against a held-out reference, crediting same-meaning answers but rejecting generic paraphrase. |

## How they fit together

- **Recall** and **recognition** define the **memory-absent** conditions (unaided
  generation vs. forced choice from a menu).
- **Familiarity** and **open-book** are **controls**: the former isolates whether the model
  knows the specific source paper; the latter isolates pure reasoning by removing the need
  to recall anything.
- The **judge** blindly scores the memory-bearing recall output against a held-out
  reference, so the recall probe can be graded for semantic equivalence rather than exact
  wording.

All five are **benchmark-agnostic templates frozen before use**: they use only generic
placeholders and neutral, made-up illustrative examples (e.g. "diffusion of heat" ↔
"spread of a rumor"), never any study pair or answer key.
