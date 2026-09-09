# Upstream fetch checkpoint — 2026-09-09

This is an incomplete refresh checkpoint, not a completed audit or release candidate.

## Boundary

The original checkout remains on `agent/fix-settings-refcell-race` at
`4e457e859db47db03d079abe64c73eda874db802` (tree
`5ecf83ee964a3e61ca5bfb7ad6049565d9fa2df6`). Its existing tracked and untracked
changes were left untouched. The isolated branch `refresh/upstreams-20260909`
is based on the existing unpublished theme release candidate
`4a533b8c4f0eea254381ef57a75d5a0f9edc127d`, which descends from the fetched
`origin/main` at `eb50c7ad`. No branch was reset or upstream content merged.

## Fetch evidence

[The pin inventory](upstream-refresh-2026-09-09-pins.json) records exact commit
and tree identities, ancestry checks, and SHA-256 hashes of raw path comparisons.
For Grok it includes every raw path in both required ranges: last reviewed to
previously fetched, and previously fetched to this run's pin. These entries are
an inventory, not a behavior classification. All Reviewed revisions remain
unchanged. Successful fetches update Latest fetched only.

The tracked Z.AI SDK remote still returns `Repository not found`; its recorded
identity is preserved. No replacement repository was inferred.

Grok is pinned to `37949780c144e37df692e3d669051a21fec24f20` (2,998 changed paths
from Reviewed; 2,665 from the previous fetch). An exhaustive behavioral review,
ports, and parity testing remain necessary. No acknowledgement is authorized.

## Bounded source inspection

- Warp, OpenCode Codex auth, Z.AI coding plugins, Z.AI usage helper, and Exa
  have unchanged reviewed heads.
- GLM-5 changes only `resources/wechat.png`; no model or API contract changed.
  Reviewed is conservatively retained at its previous pin.
- Kimi CLI legacy advances through its 1.50.0 release and deprecation-aware
  migration to the TypeScript Kimi Code application. The inspected app/shell
  changes implement migration notices and installation, outside this fork's
  provider-adapter scope. Full source review is not claimed.
- The advancing Grok, Codex, Kimi Code, OpenCode, Oh My Pi, Ghostty, Herdr,
  models.dev, and CodexBar ranges remain review queues.

## Carried obligations

The existing `fork/parity/current.json` is preserved byte-for-byte. None of its
open obligations has been closed or silently replaced. The following stable IDs
remain open, with their existing owners, blockers, acceptance tests, and deadlines:

- `CDX-E482-CONTEXT-WINDOW-ID`
- `GB-BC7F-SESSION-ACP`
- `GB-BC7F-AGENT-LIFECYCLE`
- `GB-BC7F-TUI-INPUT`
- `GB-BC7F-WORKSPACE`
- `GB-BC7F-INTEGRATION`
- `CDX-6478-AUTH-CATALOG-WIRE`
- `CDX-6478-APP-BEHAVIOR`
- `OC-DC44-CODEX-INTEROP`
- `KIMI-9D23-PROVIDER`
- `OMP-33CC-HARNESS`
- `CODEXBAR-CE44-ZAI-USAGE`

New range behaviors remain unclassified at this fetch checkpoint. This document
does not satisfy the exhaustive parity ledger requirement and must not be used
as acknowledgement evidence.

## Publication state

GitHub reports `v0.3.15` as the latest public release. No push, tag, release, or
tap mutation was performed. Homebrew is not available on this Linux aarch64
host, so the skill's brew style/audit/test and installed-upgrade checks have not
run. Publication requires completed adoption and validation first; the skill
also requires confirmation of the selected version for this run.
