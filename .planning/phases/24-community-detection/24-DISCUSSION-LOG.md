# Phase 24: Community Detection — Discussion Log

**Date:** 2026-04-10
**Format:** Full audit trail of the discuss-phase Q&A. Human reference only — downstream agents should read `24-CONTEXT.md`.

---

## Gray Area Selection

**Q:** Which gray areas for Phase 24 do you want to discuss?
**Options presented:**
1. Louvain implementation source
2. Color by dropdown scope & modes
3. Community summary panel placement
4. Summary content & 'top papers' ranking

**Selected (all four):** Louvain implementation source, Color by dropdown scope & modes, Community summary panel placement, Summary content & 'top papers' ranking

---

## Area 1: Louvain Implementation Source

**Q1.1:** Where should the Louvain modularity algorithm come from?
- Crate: graphalgs / petgraph addon (Recommended)
- In-house Louvain on petgraph
- Claude's discretion

**Answer:** Crate: graphalgs / petgraph addon
→ Locked to **D-01** (with researcher crate evaluation)

**Q1.2:** Is community detection directed or undirected?
- Undirected (treat citations symmetrically) (Recommended)
- Directed

**Answer:** Undirected
→ Locked to **D-02**

**Q1.3:** How should community detection determinism be handled?
- Fixed RNG seed (Recommended)
- Re-stable on corpus_fingerprint
- Don't stabilize

**Answer:** Fixed RNG seed
→ Locked to **D-03**

**Q1.4:** How are very small communities handled?
- Bucket as 'Other' (Recommended, threshold <3)
- Keep all communities
- Filter out singletons only

**Answer:** Bucket as 'Other' (size < 3)
→ Locked to **D-04**

**Checkpoint Q:** More questions about Louvain implementation, or move on?
**Answer:** Next area

---

## Area 2: Color by Dropdown Scope & Modes

**Q2.1:** What modes should the new 'Color by' dropdown ship with in Phase 24?
- Community + BFS Depth + Topic (full set) (Recommended)
- Community only + Uniform
- Community + BFS Depth only

**Answer:** Community + BFS Depth + Topic (full set)
→ Locked to **D-07, D-13, D-14**

**Q2.2:** Default Color by mode and interaction with Topic Rings?
- Default Community; Topic Rings stays independent (Recommended)
- Default BFS Depth; Topic Rings independent
- Default Community; Topic Rings auto-hides when ColorBy=Topic

**Answer:** Default Community; Topic Rings stays independent
→ Locked to **D-09, D-10**

**Q2.3:** Color palette choice for communities?
- Categorical palette (Tableau 10 / D3 schemeCategory10) (Recommended)
- Golden-ratio HSL generator
- Claude's discretion

**Answer:** Categorical palette
→ Locked to **D-11**

**Q2.4:** How should color transitions animate when switching Color by?
- Lerp colors (~300ms) (Recommended)
- Instant switch

**Answer:** Lerp colors (~300ms)
→ Locked to **D-12**

**Checkpoint Q:** More questions about Color by, or move on?
**Answer:** Next area

---

## Area 3: Community Summary Panel Placement

**Q3.1:** Where should the community summary panel live?
- New drawer tab (DrawerTab::Community) (Recommended)
- Dedicated Communities dashboard card
- Click-on-legend opens side panel
- Both: drawer tab + dashboard card

**Answer:** New drawer tab (DrawerTab::Community)
→ Locked to **D-15**

**Q3.2:** How does the user open a community's summary?
- Click community color in legend (Recommended)
- Auto-show when paper selected
- Both entry points

**Answer:** Both entry points
→ Locked to **D-16** (two entry paths: paper-selected and legend-click)

**Q3.3:** When a user clicks a legend chip (no paper selected), what does the drawer show?
- Community tab, no paper header (Recommended)
- Auto-select top community paper
- Dedicated community-only side panel

**Answer:** Community tab, no paper header
→ Locked to **D-17**

**Checkpoint Q:** More questions about panel placement, or move on?
**Answer:** Next area

---

## Area 4: Summary Content & Top Papers Ranking

**Q4.1:** What ranks 'top papers' within a community?
- Phase 23 PageRank restricted to community (Recommended)
- In-community citation count
- Raw global citation_count
- Hybrid: PageRank × community-internal degree

**Answer:** Hybrid: PageRank × community-internal degree
→ Locked to **D-19**

**Q4.2:** How are 'dominant keywords' for a community computed?
- Aggregated TF-IDF top-N (Recommended)
- TF-IDF with rarity boost (c-TF-IDF)
- Claude's discretion

**Answer:** TF-IDF with rarity boost (c-TF-IDF)
→ Locked to **D-21**

**Q4.3:** What does 'shared methods' mean concretely?
- Reuse shared_high_weight_terms across community
- Separate keyword bucket: method-like terms
- Drop 'shared methods' field; keep dominant keywords only (Recommended)
- Claude's discretion

**Answer:** Reuse shared_high_weight_terms across community
→ Locked to **D-23**

**Q4.4:** How many top papers + keywords to show per community?
- Top 5 papers, top 10 keywords (Recommended)
- Top 3 papers, top 8 keywords
- Top 10 papers, top 15 keywords

**Answer:** Top 5 papers, top 10 keywords
→ Locked to **D-20, D-22**

**Q4.5:** Community name/label shown to users?
- Auto from top keyword(s) (Recommended)
- 'Community N' by size rank
- Both: 'Community 1 · quantum decoherence'

**Answer:** Auto from top keyword(s)
→ Locked to **D-24**

**Checkpoint Q:** Ready to write CONTEXT.md, or explore more gray areas?
**Answer:** I'm ready for context

---

## Notable User Preferences vs. Recommendations

The user consistently overrode "recommended" defaults in favor of more sophisticated choices:
- **Q4.1:** Picked hybrid PageRank × internal-degree over the recommended simple PageRank-restricted option
- **Q4.2:** Picked c-TF-IDF over the recommended plain aggregated TF-IDF
- **Q4.3:** Picked "reuse shared_high_weight_terms" over the recommended "drop the field" — user wants the shared methods feature even if it's best-effort
- **Q3.2:** Picked "both entry points" over the recommended legend-only path — richer UX

This pattern suggests the user prefers feature-complete, discriminative implementations over minimal-viable ones for this phase.

---

## Deferred Ideas Captured

None surfaced during discussion. Deferred ideas listed in CONTEXT.md `<deferred>` section come from Claude's analysis of adjacent capabilities that are out of scope for Phase 24.
