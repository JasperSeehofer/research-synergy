import os
import sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from sme_lite import systematicity

# G_a: X1 order-param UNDERGOES-TRANSITION-AT X2 threshold; X3 coupling COUPLES X1; X4 net DEPENDS-ON X3
Ga = {"entities":[
    {"id":"a1","role":"order-parameter"},{"id":"a2","role":"threshold"},
    {"id":"a3","role":"coupling"},{"id":"a4","role":"interaction-network"}],
  "relations":[
    {"src":"a1","dst":"a2","type":"UNDERGOES-TRANSITION-AT"},
    {"src":"a3","dst":"a1","type":"COUPLES"},
    {"src":"a4","dst":"a3","type":"DEPENDS-ON"}]}
# G_b: same structure, different ids/roles-names but same roles
Gb = {"entities":[
    {"id":"b1","role":"order-parameter"},{"id":"b2","role":"threshold"},
    {"id":"b3","role":"coupling"},{"id":"b4","role":"interaction-network"}],
  "relations":[
    {"src":"b1","dst":"b2","type":"UNDERGOES-TRANSITION-AT"},
    {"src":"b3","dst":"b1","type":"COUPLES"},
    {"src":"b4","dst":"b3","type":"DEPENDS-ON"}]}
# G_c: unrelated — different relation types, disconnected
Gc = {"entities":[
    {"id":"c1","role":"state-variable"},{"id":"c2","role":"rate"},
    {"id":"c3","role":"observable"}],
  "relations":[{"src":"c1","dst":"c2","type":"INCREASES-WITH"}]}

s_ab,m_ab = systematicity(Ga,Gb,use_roles=True)
s_ac,_ = systematicity(Ga,Gc,use_roles=True)
s_aa,_ = systematicity(Ga,Ga,use_roles=True)
# roles-off: Gb with roles stripped should still match on topology
Gb_noroles = {"entities":[{"id":e["id"],"role":"x"} for e in Gb["entities"]],"relations":Gb["relations"]}
s_ab_off,_ = systematicity(Ga,Gb_noroles,use_roles=False)

print(f"identical isomorph  (expect 3): {s_ab}  map={m_ab}")
print(f"self-match          (expect 3): {s_aa}")
print(f"unrelated           (expect 0): {s_ac}")
print(f"roles-off isomorph  (expect 3): {s_ab_off}")
assert s_ab==3, "isomorph should preserve all 3 connected relations"
assert s_aa==3
assert s_ac==0
assert s_ab_off==3
# partial: Gd shares 2 of 3 relations with Ga
Gd = {"entities":[
    {"id":"d1","role":"order-parameter"},{"id":"d2","role":"threshold"},
    {"id":"d3","role":"coupling"},{"id":"d4","role":"interaction-network"}],
  "relations":[
    {"src":"d1","dst":"d2","type":"UNDERGOES-TRANSITION-AT"},
    {"src":"d3","dst":"d1","type":"COUPLES"},
    {"src":"d4","dst":"d3","type":"CAUSES"}]}  # last relation type differs
s_ad,_ = systematicity(Ga,Gd,use_roles=True)
print(f"partial (2/3 match, expect 2): {s_ad}")
assert s_ad==2, f"expected 2 got {s_ad}"
print("\nALL TOY ASSERTIONS PASS")
