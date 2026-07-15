#!/usr/bin/env python3
"""EXP-RS-20 — self-test for verify_score.py (run before spending verify tokens).

Core mechanism under test (C-40): the HyDE recall stage ranks a method-INCOHERENT distractor above
the true cross-field target (the max-pool distractor inflation that killed EXP-RS-19); the blind
verify stage marks that distractor method_coherence=false; pruning demotes it to rank inf, lifting
the true target into the top-k. Also checks: coherent candidates keep their HyDE score; pruned
candidates tie-break by candidate id lexicographic (C-40 inherits C-19); pruning the TRUE target
drops it; and default_coherent=True reproduces the un-pruned HyDE ranking.
"""
from verify_score import build_hyde, pruned_scorer, NEG_INF
from hyde_score import corpus_idf
from sme_lite import eval_direction, rank_candidates

# q = source-field query; t_b = its true cross-field analogue (partial hyp match);
# a_d1 = a method-INCOHERENT distractor the HyDE hyp matches PERFECTLY (outranks t_b); z_d2 = noise.
corpus = {"papers": [
    {"arxiv_id": "q",    "title": "Q", "abstract": "alpha beta gamma", "community_id": 0},
    {"arxiv_id": "t_b",  "title": "B", "abstract": "delta epsilon zeta", "community_id": 1},
    {"arxiv_id": "a_d1", "title": "D1", "abstract": "kappa lambda mu", "community_id": 0},
    {"arxiv_id": "z_d2", "title": "D2", "abstract": "omega psi", "community_id": 2},
]}
all_ids = [p["arxiv_id"] for p in corpus["papers"]]
idf = corpus_idf(corpus)

# q's hypotheticals: hyp0 matches a_d1 PERFECTLY (cos=1.0); hyp1 matches t_b partially (cos<1.0).
hyp = {"q": {"arxiv_id": "q", "hypotheticals": [
    {"target_field": "f0", "generic_object": "o0", "abstract": "kappa lambda mu"},   # -> a_d1 (1.0)
    {"target_field": "f1", "generic_object": "o1", "abstract": "delta epsilon"},      # -> t_b  (~0.816)
]}}
hyde_score, winning_idx, _ = build_hyde(corpus, hyp, idf, k=5)
pairs = [{"id": "toy", "side_a": "q", "side_b": "t_b"}]

# --- HyDE alone: the incoherent distractor a_d1 outranks the true target t_b ---
assert hyde_score("q", "a_d1") > hyde_score("q", "t_b") > hyde_score("q", "z_d2") == 0.0
base = eval_direction(pairs, all_ids, hyde_score, "side_a", "side_b")
assert base["ranks"]["toy"] == 2, f"expected t_b at rank 2 pre-prune, got {base['ranks']}"
assert base["recall@1"] == 0.0
# winning hypothetical for t_b is the b-vocab one (idx 1), for a_d1 the perfect one (idx 0)
assert winning_idx("q", "t_b") == 1 and winning_idx("q", "a_d1") == 0

# --- verify prunes the incoherent distractor -> target lifts to rank 1 (the cascade's whole point) ---
verdicts = {
    ("q", "a_d1"): {"method_coherence": False, "object_difference": True, "rationale": ""},
    ("q", "t_b"):  {"method_coherence": True,  "object_difference": True, "rationale": ""},
    ("q", "z_d2"): {"method_coherence": True,  "object_difference": True, "rationale": ""},
}
psc = pruned_scorer(hyde_score, verdicts)
assert psc("q", "a_d1") == NEG_INF, "pruned distractor must score -inf (C-40)"
assert psc("q", "t_b") == hyde_score("q", "t_b"), "coherent candidate must keep its HyDE score"
pruned = eval_direction(pairs, all_ids, psc, "side_a", "side_b")
assert pruned["ranks"]["toy"] == 1, f"pruning must lift t_b to rank 1, got {pruned['ranks']}"
assert pruned["recall@1"] == 1.0

# --- C-40 tie-break: multiple pruned candidates order by candidate id lexicographic ---
v2 = dict(verdicts)
v2[("q", "z_d2")] = {"method_coherence": False, "object_difference": True, "rationale": ""}
ranked = rank_candidates("q", all_ids, pruned_scorer(hyde_score, v2))
order = [cid for cid, _ in ranked]
assert order == ["t_b", "a_d1", "z_d2"], f"pruned tie-break not lexicographic: {order}"

# --- over-pruning: marking the TRUE target incoherent drops it out of the top ---
v3 = {("q", "a_d1"): {"method_coherence": True, "object_difference": True, "rationale": ""},
      ("q", "t_b"):  {"method_coherence": False, "object_difference": True, "rationale": ""},
      ("q", "z_d2"): {"method_coherence": True, "object_difference": True, "rationale": ""}}
dropped = eval_direction(pairs, all_ids, pruned_scorer(hyde_score, v3), "side_a", "side_b")
assert dropped["ranks"]["toy"] == 3, f"a pruned target must fall to last, got {dropped['ranks']}"

# --- default_coherent=True with no verdicts reproduces the un-pruned HyDE ranking ---
nopruned = eval_direction(pairs, all_ids, pruned_scorer(hyde_score, {}, default_coherent=True),
                          "side_a", "side_b")
assert nopruned["ranks"] == base["ranks"], "empty verdicts (default coherent) must equal HyDE-alone"

print("verify_toytest: all assertions passed")
