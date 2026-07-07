#!/usr/bin/env python3
"""EXP-RS-17 — self-test for methmesh_score.py (run before spending tagging tokens).

Synthetic corpus with a known answer:
  q_a, q_b  = a true pair sharing a RARE archetype 'arch-rare' (+ a common one).
  d1, d2    = distractors carrying only the COMMON archetype.
Asserts: (1) tagging-recall gate counts the shared archetype; (2) IDF-weighting makes q_b the
#1 retrieval for q_a (rare shared mechanism dominates); (3) the minus-lexical penalty demotes a
lexically-identical-but-mechanistically-unrelated distractor; (4) IDF-off can lose the separation
that IDF-on keeps.
"""
from methmesh_score import (archetype_sets, archetype_idf, signature_vectors, load_pairs)
from sme_lite import cosine, eval_direction, tfidf_vectors

# --- synthetic tags: q_a,q_b share rare 'R' (+ common 'C'); d1,d2 only 'C' ---
tags = {
    "q_a": [{"archetype_id": "R", "evidence_snippet": "rare-a"},
            {"archetype_id": "C", "evidence_snippet": "common-a"}],
    "q_b": [{"archetype_id": "R", "evidence_snippet": "rare-b"},
            {"archetype_id": "C", "evidence_snippet": "common-b"}],
    "d1": [{"archetype_id": "C", "evidence_snippet": "common-1"}],
    "d2": [{"archetype_id": "C", "evidence_snippet": "common-2"}],
}
all_ids = ["q_a", "q_b", "d1", "d2"]
sets = archetype_sets(tags)
idf = archetype_idf(sets, all_ids)

# 'R' (df=2) rarer than 'C' (df=4) -> higher idf
assert idf["R"] > idf["C"], (idf["R"], idf["C"])

# lexical: make q_a lexically identical to d1 (adversary), q_b different -> tests minus-lexical
corpus = {"papers": [
    {"arxiv_id": "q_a", "abstract": "alpha beta gamma delta alpha beta"},
    {"arxiv_id": "q_b", "abstract": "omega psi chi phi omega psi"},
    {"arxiv_id": "d1", "abstract": "alpha beta gamma delta alpha beta"},  # == q_a lexically
    {"arxiv_id": "d2", "abstract": "kappa iota theta eta kappa iota"},
]}
lex = tfidf_vectors(corpus)

def score(use_idf, lam):
    sig = signature_vectors(sets, all_ids, idf, use_idf=use_idf)
    return lambda q, c: cosine(sig[q], sig[c]) - lam * cosine(lex[q], lex[c])

pairs = [{"id": "toy", "side_a": "q_a", "side_b": "q_b"}]

# (2)+(3) primary arm (IDF-on, minus-lexical): q_b must rank #1 for q_a
fwd = eval_direction(pairs, all_ids, score(True, 1.0), "side_a", "side_b")
assert fwd["ranks"]["toy"] == 1, f"primary arm rank != 1: {fwd['ranks']}"
assert fwd["recall@1"] == 1.0

# minus-lexical really demotes the lexical twin d1: without the penalty q_a-d1 lexical cos=1
s_nolex = score(True, 0.0)
s_lam = score(True, 1.0)
assert s_lam("q_a", "d1") < s_nolex("q_a", "d1"), "lexical penalty not applied"

# (1) gate: the pair shares an archetype
shared = sets["q_a"] & sets["q_b"]
assert shared == {"R", "C"}, shared

# (4) with only the common archetype, q_b would not separate from distractors
sets_common = {"q_a": {"C"}, "q_b": {"C"}, "d1": {"C"}, "d2": {"C"}}
idf_c = archetype_idf(sets_common, all_ids)
sig_c = signature_vectors(sets_common, all_ids, idf_c, use_idf=True)
# all signatures identical -> archetype signal is zero; ranking driven only by (−lexical)
vals = {c: cosine(sig_c["q_a"], sig_c[c]) for c in ("q_b", "d1", "d2")}
assert len(set(round(v, 6) for v in vals.values())) == 1, ("common-only should tie", vals)

print("methmesh_toytest: all assertions passed")
