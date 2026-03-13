# TODO

## Completed
- [x] implement HTML access and parser for arxiv
- [x] Phase 0: Foundation (bug fixes, error handling, async cleanup, tests, CI)

## Phase 1: SurrealDB + InspireHEP
- [ ] Add SurrealDB persistence (paper cache, citation edges)
- [ ] Implement InspireHEP API integration ([api docs](https://github.com/inspirehep/rest-api-doc))
- [ ] Create `DataSource` trait abstracting arXiv vs InspireHEP
- [ ] Build graph from DB with SurrealDB graph traversal

## Phase 2: Leptos Webapp
- [ ] Set up Cargo workspace (core, aggregation, database, app, frontend, server)
- [ ] Leptos 0.8 + Axum 0.8 setup
- [ ] Server functions (crawl, search, paper details, citation graph)
- [ ] SVG force-directed graph visualization
- [ ] Pages: Home, Paper, Graph

## Phase 3: Production
- [ ] Playwright E2E tests
- [ ] Server rate limiting
- [ ] Docker + docker-compose
- [ ] Updated CI for webapp
