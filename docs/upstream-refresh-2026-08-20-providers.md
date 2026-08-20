# Provider and prior-refresh audit — 2026-08-20

This audit accompanies the Grok Build `19d42e35c07a9c9244f03f6df0c4c353f970d4f9` adoption. It reviews every configured inspiration source inside the provider-adapter, interoperability, or research scope defined by `AGENTS.md`; it does not treat those projects as alternative application architectures.

## Prior-refresh residue

All retained refresh worktrees were inspected before the new audit began. The older refresh worktrees were clean except `grok-build-enhanced-refresh-20260809`, which contains 153 staged paths, ten unstaged paths, and one untracked ledger file. It is intentionally preserved as forensic evidence.

Comparing its 160 distinct paths against `origin/main` showed that 153 exact blobs are already reachable. Five remaining paths are superseded refresh evidence or manifest drafts. Two user-guide pages, `23-dashboard.md` and `24-monitoring-usage.md`, were present in the repository but absent from the guide index. This refresh links both pages and the new status-line page from `docs/user-guide/README.md`. No runtime implementation, provider behavior, or open parity obligation was stranded in an older worktree.

The cumulative obligation ledger remains append-only: 120 obligations are closed, eight credential/network-gated obligations remain `offline-qualified`, and no obligation is open or temporarily deferred.

## Immutable fetched identities

| Source | Previous reviewed | Latest fetched |
| --- | --- | --- |
| OpenAI Codex | `c4941302c73c6322b153bba13ac0a9f4396301d6` | `478dbe9df0a33141d265db5977947cc432e7fe85` |
| OpenCode | `284214c78d32a09fd9c729bdefc07be50f74eb40` | `b155b15694dbcc6768f11d2f25cc2bdd1f738ab4` |
| Grok Build | `9fabadea800fa6e2ed8ec91c4f45f02b7e2504f4` | `19d42e35c07a9c9244f03f6df0c4c353f970d4f9` |
| OpenCode Codex auth | `bec2ad69b252ef4ad7dd33b9532ff8b4fdb6d016` | unchanged |
| Oh My Pi | `0e8142ad0e3189b5b51b49fd3434354683ba1b01` | `72000acfeb902e21816252699482887f34d1a5a4` |
| Warp themes | `82e51dcf9b47912d551107748ba3297a21b2eff3` | unchanged |
| Kimi Code | `437a1b8ba1b7e0f6662bdadc669564fdc58c3f5a` | `3899079a2c851bd0b3f1cbf1d3d2fd9026fc6abb` |
| Kimi CLI | `cbc15c076d17f70fec9f89c90c0502e68657f505` | unchanged |
| Z.AI SDK | `ca5109c0aa9bf173839be391b4b14aeadf9a9bf9` | retained; remote fetch unavailable |
| Z.AI coding plugins | `0446d0bb0bc537d97d3ab3664c4b8b9c4a0e1254` | unchanged |
| GLM-5 | `436efa09bc868a6922e307624189e7018406beb9` | `25206af860c4ac10f6411c597c574f9b1c00e53c` |
| CodexBar | `22b24b885693e890af52df15c29f7ca024904c74` | `e92d89ab20037e4b3ba2028e22a4f5aef2929ab1` |
| Z.AI usage helper | `54cd1f33a703c417f2492ee1f21f22b3633a43c4` | unchanged |
| models.dev | `ac01bd90859928691e2e8e65df5cf390ffb1539e` | `4b494b2702c0155e596765ea3f0d007a688f7fbb` |
| Exa MCP server | `394f9210ed16d3e25d328e1e6db285824caedc04` | `66bacbe4afd35a7e1671be9ab55c2b6bf60aff34` |

## Changed-source inventory

Hashes are SHA-256 over exact `git diff --no-renames --raw --abbrev=40` output.

| Source | Commits | Raw paths | Target tree | Raw hash |
| --- | ---: | ---: | --- | --- |
| OpenAI Codex | 248 | 1,165 | `495e563c062eb2bf99772be4e8a6b05e795c87c0` | `976ffe43bb1f9e49fbb2075df4816b20281c43cacbd1ee6a39f30d0759073783` |
| OpenCode | 114 | 214 | `f67dcb90e9fe1fc591c68dbfcc0cafdb8af6b7ee` | `0dfadee673b103cd7ada2bc4763485d04ecb904378bcaa9fe27c7d1934949377` |
| Oh My Pi | 1,234 | 2,339 | `c337de936afe886b9c5d566d026427839462dbf4` | `b7119d0af3e0a3cd4108ec0bd14faa9a906267a0e7e2ea21bedc965afeaea30b` |
| Kimi Code | 138 | 2,194 | `345d6bfe77a49af3d6d2c7431540c20f346062c5` | `bedd2d02a6b67d3090eaeefc25f84ffd91e7f32725595b0e19db11a394e89e10` |
| GLM-5 | 1 | 6 | `573d8342bcfc2e21d27e210c47a99a4604fc39ee` | `3a93baf46a08a39a94b571232ea58e5e9de140b36d3d192b8725799a3ca29266` |
| CodexBar | 716 | 871 | `1ef609d6d0c9cf15ffe2f2e2080931513bb9b3f4` | `a22bde5a1438fe6492a7574b2f7197f7c962ac4a8ce125d8e4c74fce216c73ae` |
| models.dev | 711 | 1,658 | `0aaf9201e5c573d912279ee431dcc45bf970deb7` | `a9e76b20812885638ed0c7e3d842adaf4a89654545684a7eb0e2972c21abd0e5` |
| Exa MCP server | 4 | 14 | `e8d126cfd95c5b1e9e9d2a8cb33e587e81de4d29` | `b2d597660d8f353af848fcd3e7c0ce27c2f3d910699aac1bcc316f7117fd4bec` |

## Provider classification

### OpenAI Codex

- **Adopt:** present `edu_plus` and `edu_pro` as friendly public plan labels; derive an optional provider-managed compute-residency header from the access-token claim, mark it sensitive, include it in route/catalog identity, and remove stale residency on credential change.
- **Already equivalent:** provider-authoritative context windows and catalog metadata; stable Codex turn state; account- and generation-bound request auth; invalid-grant recovery; `ultra` reasoning; authoritative subscription/thread usage; prompt-cache and compaction compatibility; and ambient-auth isolation.
- **Not applicable:** Codex app-server, TUI, Guardian, workload-identity, gRPC, persistent-exec, sandbox, WebSocket, and official release architecture. Enhanced keeps Grok's application and SSE sampler surfaces.

The residency header is produced only by `CodexAuthSnapshot`; user/model extra headers cannot override it. Authorization, account, FedRAMP, and residency values are sensitive `HeaderValue`s, and route hashing changes when residency changes without exposing the claim.

### OpenCode and OpenCode Codex auth

- **Already equivalent:** provider-scoped authenticated catalog replacement, bounded provider-defined reasoning fields, credential-generation invalidation, and no fallback to xAI or a static API key.
- **Not applicable:** OpenCode OAuth application flows and arbitrary environment-derived provider headers. Direct Codex login remains Enhanced's isolated subscription adapter, and accepting generic credential headers would weaken its wire contract.

### Kimi Code and Kimi CLI

- **Adopt/already equivalent:** typed empty-output failures, provider-hosted web behavior, bounded reasoning fields, API-key catalog discovery, video upload/projection, and ZDR storage-error presentation.
- **Not applicable:** Kimi's application server, OAuth UI, CLI resume commands, and arbitrary custom authentication headers. Enhanced intentionally remains an API-key provider adapter.

### Other inspiration and research sources

- **Oh My Pi:** no normative application behavior; its large harness delta remains evaluation-only.
- **Warp themes:** unchanged and still locked by the vendor corpus checker.
- **GLM-5, Z.AI SDK/plugins/helper, and CodexBar:** research only; no Z.AI runtime provider, login, credentials, or product claim is established.
- **models.dev:** third-party catalog rows remain hints; authenticated first-party catalogs are authoritative.
- **Exa MCP server:** container and packaging changes do not alter Enhanced's MCP client contract.

## Outcome

Every fetched delta is classified as adopted, already equivalent, or not applicable under a standing scope or credential-safety rule. No new provider deferral is opened. The eight existing offline-qualified tests remain explicit and are carried forward unchanged because they require entitled credentials or live provider routes; they are not Grok behavior-adoption deferrals.
