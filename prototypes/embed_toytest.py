#!/usr/bin/env python3
"""EXP-RS-21 per-model liveness self-test (NON-benchmark; safe to run pre-LOCK).

Certifies EACH reproducible local encoder (bge, gte, specter) individually (MUST-9), on a tiny
synthetic {title, abstract} corpus, that:
  (1) the encoder loads and produces L2-normed, non-degenerate vectors of the right shape,
  (2) a near-paraphrase of the query is retrieved at rank 1 among topical distractors,
  (3) a cross-vocabulary analogue (percolation<->epidemic) outranks pure-noise distractors.
Writes data/embed_toytest_pass.json = {bge:bool, gte:bool, specter:bool} — the liveness attestation
embed_gate.py consumes. Green (all True) before any benchmark run. Does NOT touch the benchmark corpora.
"""
import json
import os
import sys

import numpy as np

from embed_score import encode_st, encode_specter, MODEL_SPECS
from sme_lite import eval_direction

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")

PAPERS = [
    {"arxiv_id": "toy-perc", "title": "Percolation threshold on random graphs",
     "abstract": "We study bond percolation on random graphs and locate the critical threshold at "
                 "which a giant connected cluster emerges. Near the threshold the cluster-size "
                 "distribution follows a branching process governed by the mean degree."},
    {"arxiv_id": "toy-perc-para", "title": "Critical connectivity transition in random networks",
     "abstract": "Bond occupation on random networks exhibits a critical point where a giant "
                 "connected component first appears. The size of clusters below threshold obeys a "
                 "branching-process law set by the average node degree."},
    {"arxiv_id": "toy-epi", "title": "Epidemic spreading and the outbreak threshold on contact networks",
     "abstract": "An infectious disease spreads over a contact network. Below a basic reproduction "
                 "number of one the outbreak dies out; above it a macroscopic fraction of the "
                 "population is infected. The final outbreak size is computed from a branching process."},
    {"arxiv_id": "toy-cook", "title": "A slow-braised beef stew recipe",
     "abstract": "This recipe braises chuck beef with red wine, carrots, and thyme for three hours "
                 "until tender. Serve over mashed potatoes with a crusty baguette."},
    {"arxiv_id": "toy-music", "title": "Counterpoint techniques in Baroque keyboard fugues",
     "abstract": "We analyze voice-leading and subject-answer relationships in Bach's keyboard "
                 "fugues, focusing on stretto, inversion, and pedal points in the final entries."},
]
ALL = [p["arxiv_id"] for p in PAPERS]


def test_model(mk):
    hf_id, kind = MODEL_SPECS[mk]
    enc = encode_specter if kind == "specter" else encode_st
    vecs, rev, seq, _ = enc(hf_id, PAPERS, mk)
    assert vecs.shape[0] == len(PAPERS), "bad row count"
    assert np.allclose(np.linalg.norm(vecs, axis=1), 1.0, atol=1e-5), "not L2-normed"
    assert np.max(np.std(vecs, axis=0)) > 1e-6, "degenerate embeddings"
    emb = {a: vecs[i] for i, a in enumerate(ALL)}

    def sc(a, b):
        return float(np.dot(emb[a], emb[b]))

    r_para = eval_direction([{"id": "para", "side_a": "toy-perc", "side_b": "toy-perc-para"}],
                            ALL, sc, "side_a", "side_b")["ranks"]["para"]
    epi = sc("toy-perc", "toy-epi")
    noise = [sc("toy-perc", "toy-cook"), sc("toy-perc", "toy-music")]
    ok = (r_para == 1) and (epi > max(noise))
    print(f"  {mk:9} rev={str(rev)[:34]:34} para_rank={r_para} "
          f"cos(perc,epi)={epi:.3f} cos(noise)={[round(x,3) for x in noise]} -> "
          f"{'PASS' if ok else 'FAIL'}")
    return ok


def run():
    passes = {}
    for mk in ("bge", "gte", "specter"):
        try:
            passes[mk] = bool(test_model(mk))
        except Exception as e:  # noqa: BLE001
            print(f"  {mk:9} ERROR {type(e).__name__}: {e}")
            passes[mk] = False
    os.makedirs(DATA, exist_ok=True)
    json.dump(passes, open(os.path.join(DATA, "embed_toytest_pass.json"), "w"), indent=1)
    all_ok = all(passes.values())
    print(f"\nattestation -> data/embed_toytest_pass.json = {passes}")
    print("==> TOYTEST", "PASS (all encoders live)" if all_ok else "FAIL")
    return all_ok


if __name__ == "__main__":
    sys.exit(0 if run() else 1)
