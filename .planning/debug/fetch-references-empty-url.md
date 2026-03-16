---
status: resolved
trigger: "fetch_references fails with HTML download error: : builder error — empty URL"
created: 2026-03-16T00:00:00Z
updated: 2026-03-16T00:00:00Z
---

## Current Focus

hypothesis: convert_pdf_url_to_html_url receives an empty pdf_url from Paper, producing an empty string passed to reqwest, which returns a "builder error"
test: traced full call chain from crawl.rs -> fetch_references -> aggregate_references_for_arxiv_paper -> convert_pdf_url_to_html_url -> downloader
expecting: confirmed — the empty string URL is the direct cause of reqwest failing at the client.get() call
next_action: DONE — root cause confirmed, no fix applied (investigate-only mode)

## Symptoms

expected: fetch_references crawls the HTML page at https://arxiv.org/html/2503.18887 and returns references
actual: WARN "HTML download failed for : builder error" — the URL is empty, reqwest builder cannot construct a request for ""
errors: |
  WARN resyn_core::data_aggregation::html_parser: HTML download failed for : builder error
  WARN resyn::commands::crawl: Failed to fetch references paper_id="2503.18887" error=HTML download error: : builder error
reproduction: run `resyn crawl -p 2503.18887` with arxiv source
started: unknown — likely always present when pdf_url is empty on a Paper

## Eliminated

- hypothesis: ArxivHTMLDownloader is not initialized properly
  evidence: make_source() in crawl.rs creates it correctly via ArxivHTMLDownloader::new(client).with_rate_limit(Duration::ZERO), and the client is valid (create_http_client() always succeeds)
  timestamp: 2026-03-16

- hypothesis: rate_limiter interaction corrupts the downloader
  evidence: rate limiter is a separate SharedRateLimiter; wait_for_token() is called before source.fetch_paper(), not inside fetch_references. The ArxivHTMLDownloader has its own internal rate limit set to ZERO. No interaction.
  timestamp: 2026-03-16

## Evidence

- timestamp: 2026-03-16
  checked: html_parser.rs download_html()
  found: |
    Line 56: warn!("HTML download failed for {html_url}: {e}");
    This is the exact line producing the log message. The {html_url} interpolation produces "" (empty),
    confirming the URL passed to this function is "".
  implication: The URL arrives at download_html as an empty string.

- timestamp: 2026-03-16
  checked: arxiv_utils.rs aggregate_references_for_arxiv_paper()
  found: |
    Line 14: let html_url = convert_pdf_url_to_html_url(&paper.pdf_url);
    The URL is derived entirely from paper.pdf_url. No validation or fallback.
  implication: If paper.pdf_url is "", html_url will also be "".

- timestamp: 2026-03-16
  checked: arxiv_utils.rs convert_pdf_url_to_html_url()
  found: |
    fn convert_pdf_url_to_html_url(pdf_url: &str) -> String {
        pdf_url.replace(".pdf", "").replace("pdf", "html")
    }
    Input "" -> replace(".pdf", "") -> "" -> replace("pdf", "html") -> "".
    The function returns "" when given "".
  implication: No guard against empty input.

- timestamp: 2026-03-16
  checked: paper.rs Paper struct and from_arxiv_paper()
  found: |
    pdf_url is a plain String with Default = "".
    from_arxiv_paper() sets pdf_url: arxiv_paper.pdf_url.clone() — directly from the arxiv-rs Arxiv struct.
    If arxiv-rs returns an Arxiv with pdf_url = "", the Paper will have pdf_url = "".
  implication: The arxiv-rs library is the upstream source of the empty pdf_url.

- timestamp: 2026-03-16
  checked: arxiv_api.rs get_paper_by_id()
  found: |
    Uses ArxivQueryBuilder::new().id_list(id).build() then arxiv::fetch_arxivs().
    The arxiv-rs Arxiv struct is accepted as-is; no validation of pdf_url before passing to Paper::from_arxiv_paper.
  implication: No defensive check between arxiv-rs and the HTML downloader call.

- timestamp: 2026-03-16
  checked: crawl.rs make_source() and the spawn closure
  found: |
    fetch_paper is called first (line 316). On success, fetch_references is called (line 325).
    fetch_references failure is logged as a warning but does NOT abort the paper upsert.
    The paper is still stored with empty references.
  implication: The empty-URL failure is non-fatal; it's a degraded-mode outcome, not a crash.

## Resolution

root_cause: |
  paper.pdf_url is empty ("") for some papers returned by the arxiv-rs library.
  aggregate_references_for_arxiv_paper() passes paper.pdf_url directly to convert_pdf_url_to_html_url()
  with no guard for the empty case. That function returns "" unchanged.
  ArxivHTMLDownloader.download_html("") calls reqwest::Client::get("") which fails immediately
  with "builder error" because "" is not a valid URL.
  The resulting ResynError::HtmlDownload("": builder error) is logged with the format
  "HTML download failed for {html_url}: {e}" where html_url="" and e="builder error",
  producing the observed log line "HTML download failed for : builder error".

fix: not applied (investigate-only)
verification: not applicable
files_changed: []
