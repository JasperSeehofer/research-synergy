---
status: diagnosed
trigger: "Investigate why `cargo run --bin resyn -- crawl status --db surrealkv://./test_data` fails with 'error: unexpected argument '--db' found'"
created: 2026-03-16T00:00:00Z
updated: 2026-03-16T00:00:00Z
---

## Current Focus

hypothesis: confirmed — clap's subcommand parsing consumes remaining args once a subcommand is matched, so --db after `status` is never seen by CrawlArgs
test: static code analysis of CrawlArgs and CrawlSubcommand definitions
expecting: subcommand variants have no fields, so clap rejects any trailing args
next_action: report root cause (diagnose-only mode)

## Symptoms

expected: `resyn crawl status --db surrealkv://./test_data` connects to the given DB and prints queue counts
actual: clap exits with "error: unexpected argument '--db' found"
errors: error: unexpected argument '--db' found
reproduction: `cargo run --bin resyn -- crawl status --db surrealkv://./test_data`
started: always — by design of the current arg structure

## Eliminated

- hypothesis: --db not defined on CrawlArgs at all
  evidence: CrawlArgs has `pub db: String` at line 64 of crawl.rs — it is defined
  timestamp: 2026-03-16

- hypothesis: --db propagation issue between parent command and subcommand
  evidence: there is no separate parent command; CrawlArgs IS the args struct for the crawl subcommand — no nesting needed at that layer
  timestamp: 2026-03-16

## Evidence

- timestamp: 2026-03-16
  checked: resyn-server/src/commands/crawl.rs lines 34-42 (CrawlSubcommand enum)
  found: CrawlSubcommand::Status, ::Clear, ::Retry are unit variants — they carry zero fields and define zero clap arguments
  implication: once clap matches the `status` token as a subcommand, it enters the Status variant's argument parser; that parser accepts nothing, so `--db` is unknown to it

- timestamp: 2026-03-16
  checked: resyn-server/src/commands/crawl.rs lines 44-97 (CrawlArgs struct)
  found: `--db` is defined as a field of CrawlArgs (line 64), and `subcmd: Option<CrawlSubcommand>` is defined with `#[command(subcommand)]` (lines 94-96)
  implication: clap parses CrawlArgs fields first, then dispatches remaining tokens to the matched subcommand; but the user put --db AFTER the subcommand name, which means clap has already entered the subcommand context and CrawlArgs' own arg scanner is no longer active

- timestamp: 2026-03-16
  checked: clap subcommand argument scoping rules
  found: in clap, once a subcommand token is consumed, all subsequent tokens belong to that subcommand's parser — the parent (CrawlArgs) no longer processes them; `--db` after `status` is therefore presented to CrawlSubcommand::Status, which has no fields and thus rejects it
  implication: argument ORDER matters — `--db` must come before the subcommand name to be parsed by CrawlArgs

## Resolution

root_cause: |
  CrawlSubcommand variants (Status, Clear, Retry) are unit structs — they define no arguments
  of their own. When the user writes `crawl status --db <val>`, clap's parser:
    1. matches `crawl` → enters CrawlArgs parser
    2. matches `status` → dispatches into CrawlSubcommand::Status parser
    3. presents `--db` to CrawlSubcommand::Status — which has no arguments defined
    4. clap errors: "unexpected argument '--db' found"

  The `--db` flag IS defined on CrawlArgs, but clap stops scanning CrawlArgs arguments
  the moment it recognises a subcommand token. Everything after that token is owned by
  the subcommand. Because the subcommand variants have no fields, any trailing flag is
  rejected.

  The flag would be accepted if placed BEFORE the subcommand name:
    `resyn crawl --db surrealkv://./test_data status`   ← works
    `resyn crawl status --db surrealkv://./test_data`   ← fails (current user expectation)

fix: not applied (diagnose-only mode)
verification: n/a
files_changed: []

## Fix Directions (for reference)

Two viable approaches:

1. **Move --db before the subcommand (documentation / UX change only)**
   Requires no code change. User must write:
     `resyn crawl --db surrealkv://./test_data status`
   Downsides: unintuitive; breaks the "subcommand then its flags" mental model.

2. **Add --db to each CrawlSubcommand variant**
   Change unit variants to named structs carrying their own `db` field:
   ```rust
   #[derive(Subcommand, Debug)]
   pub enum CrawlSubcommand {
       Status { #[arg(long, default_value = "surrealkv://./data")] db: String },
       Clear  { #[arg(long, default_value = "surrealkv://./data")] db: String },
       Retry  { #[arg(long, default_value = "surrealkv://./data")] db: String },
   }
   ```
   Then read `db` from the variant in the `run()` match block rather than from
   `args.db`. This is the cleanest user-facing fix; it matches the expectation that
   `crawl status --db <val>` works.

3. **Use clap's `#[command(flatten)]` or global arg approach**
   Pull --db out into a shared `DbArgs` struct and flatten it into both CrawlArgs
   and each subcommand, or mark it as a global arg — but clap's "global" args have
   their own scoping nuances that require care.
