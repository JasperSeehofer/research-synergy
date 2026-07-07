#!/usr/bin/env python3
"""EXP-RS-19 — self-test for hyde_score.py (run before spending generation tokens).

Core mechanism under test: a QUERY q and its true cross-field analogue b share NO abstract tokens
(a vocabulary gap the C-17 lexical null cannot cross), but a generated hypothetical for q is written
in b's vocabulary -> HyDE retrieves b at rank 1 where the lexical null fails. Also checks max-pool
over K and OOV-token dropping.
"""
from hyde_score import corpus_idf, build_scorer, hyp_vec
from sme_lite import tfidf_vectors, cosine, eval_direction

corpus = {"papers": [
    {"arxiv_id": "q",  "abstract": "alpha beta gamma alpha beta"},          # source-field vocab
    {"arxiv_id": "b",  "abstract": "delta epsilon zeta delta epsilon"},     # target-field analogue (disjoint from q)
    {"arxiv_id": "d1", "abstract": "omega psi chi omega psi"},
    {"arxiv_id": "d2", "abstract": "kappa iota theta kappa iota"},
]}
all_ids = [p["arxiv_id"] for p in corpus["papers"]]
idf = corpus_idf(corpus)

# q's hypotheticals: one in b's vocabulary (the analogue field), one that matches nothing + OOV noise
hyp = {"q": {"arxiv_id": "q", "hypotheticals": [
    {"target_field": "field-b", "generic_object": "obj", "abstract": "delta epsilon zeta phenomenon"},
    {"target_field": "field-x", "generic_object": "obj", "abstract": "nonexistentword anotheroov qqzz"},
]}}

# lexical null: q vs b share no tokens -> object_sim = 0 (the gap HyDE must cross)
lex = tfidf_vectors(corpus)
assert cosine(lex["q"], lex["b"]) == 0.0, "toy setup broken: q and b should be lexically disjoint"

pairs = [{"id": "toy", "side_a": "q", "side_b": "b"}]

# headline HyDE (lam=0, k=5, max): b must rank #1 for q via the hypothetical
sc = build_scorer(corpus, hyp, idf, lam=0.0, k=5, pool="max")
fwd = eval_direction(pairs, all_ids, sc, "side_a", "side_b")
assert fwd["ranks"]["toy"] == 1, f"HyDE failed to bridge the vocab gap: {fwd['ranks']}"
assert fwd["recall@1"] == 1.0

# max-pool picks the b-matching hypothetical over the noise one
assert sc("q", "b") > 0.0
assert sc("q", "d1") == 0.0 and sc("q", "d2") == 0.0, "noise hypothetical leaked onto distractors"

# OOV tokens dropped: the 2nd hypothetical's words are absent from the corpus IDF -> empty vec
v_oov = hyp_vec(hyp["q"]["hypotheticals"][1], idf)
assert v_oov == {}, f"OOV not dropped: {v_oov}"

# lambda subtracts the object (lexical) similarity
sc_lam = build_scorer(corpus, hyp, idf, lam=1.0, k=5, pool="max")
# with q,b lexically disjoint, object_sim=0, so lambda doesn't change q-b; check on a lexical twin
corpus2 = {"papers": corpus["papers"] + [{"arxiv_id": "twin", "abstract": "alpha beta gamma alpha beta"}]}
idf2 = corpus_idf(corpus2)
sc0 = build_scorer(corpus2, hyp, idf2, lam=0.0)
sc1 = build_scorer(corpus2, hyp, idf2, lam=1.0)
assert sc1("q", "twin") < sc0("q", "twin"), "lambda did not penalize the lexical twin"

print("hyde_toytest: all assertions passed")
