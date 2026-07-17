# Third-party vendored source and data

This repository holds **upstream source and data** vendored for deterministic
builds. It is **not** first-party application code. Most projects live in this
directory; package-time data may live below its consuming crate when Cargo
requires every build input to remain inside that crate's package root.

## Why vendor

These crates sit on the path that renders **untrusted model output** (diagram
source → SVG). Vendoring gives a full audit surface, pins exact source, and
avoids crates.io yanks. Local patches and upgrade checklists live in each
crate’s `Cargo.toml` header comments — treat those as the source of truth when
re-vendoring.

## Mermaid layout stack

| Crate | Version | License | Upstream | Full license text |
|-------|---------|---------|----------|-------------------|
| [`mermaid-to-svg`](./mermaid-to-svg/) | (path) | MIT | [warpdotdev/mermaid-to-svg](https://github.com/warpdotdev/mermaid-to-svg) | [`LICENSE`](./mermaid-to-svg/LICENSE) |
| [`dagre_rust`](./dagre_rust/) | 0.0.5 | Apache-2.0 | [r3alst/dagre-rust](https://github.com/r3alst/dagre-rust) / Warp re-vendor | [`LICENCE`](./dagre_rust/LICENCE) |
| [`graphlib_rust`](./graphlib_rust/) | 0.0.2 | Apache-2.0 | [r3alst/graphlib-rust](https://github.com/r3alst/graphlib-rust) | [`LICENCE`](./graphlib_rust/LICENCE) |
| [`ordered_hashmap`](./ordered_hashmap/) | 0.0.3 | Apache-2.0 | [r3alst/ordered-hashmap](https://github.com/r3alst/ordered-hashmap) | [`LICENCE`](./ordered_hashmap/LICENCE) |

Dependency shape:

```text
xai-grok-mermaid
  └── mermaid-to-svg          (MIT)
        ├── dagre_rust        (Apache-2.0)
        │     ├── graphlib_rust
        │     └── ordered_hashmap
        └── graphlib_rust     (Apache-2.0)
              └── ordered_hashmap
```

## Notices and ancestry

- **[`NOTICE`](./NOTICE)** — short index of the crates above (names, licenses,
  upstream links, paths to full text). Prefer that file for a one-page overview.
- **[`mermaid-to-svg/THIRD_PARTY_NOTICES`](./mermaid-to-svg/THIRD_PARTY_NOTICES)** —
  additional ancestry for the SVG engine (e.g. mermaid.js, dagre.js MIT notices).

British spelling **`LICENCE`** is intentional on the Apache crates (as upstream
vendored); grepping only for `LICENSE` will miss them.

## crates.io dependencies

Normal Cargo dependencies (tokio, serde, …) are **not** under `third_party/`.
They resolve via `Cargo.lock` / crates.io. Full attribution and license texts
for the Grok CLI dependency closure are maintained in
[`THIRD-PARTY-NOTICES`](../THIRD-PARTY-NOTICES).

Only **in-tree vendored** projects are indexed here; ordinary Cargo
dependencies continue to resolve from `Cargo.lock`.

## Packaged theme data

The complete pinned set of 340 YAML/YML definitions from
[`warpdotdev/themes`](https://github.com/warpdotdev/themes) lives at
[`xai-grok-pager-render/assets/warp-themes`](../crates/codegen/xai-grok-pager-render/assets/warp-themes/).
Keeping it inside `xai-grok-pager-render` makes `cargo package` self-contained.
Its upstream Apache-2.0 license, revision marker, preserved per-theme comments,
MIT-derived Cherry/Soft Tactile attribution, and source-dependency credits are
kept alongside the corpus. See [`NOTICE`](./NOTICE) and the root
[`THIRD-PARTY-NOTICES`](../THIRD-PARTY-NOTICES) for redistribution details.

## Upgrading

1. Read the `VENDORING NOTES` block at the top of the crate’s `Cargo.toml`.
2. Re-apply listed local patches (fmt, hermetic env, unsafe fixes, dropped bins/tests).
3. Confirm the license file still matches the declared `license =` field.
4. Refresh [`NOTICE`](./NOTICE) if versions or upstream URLs change.

For the Warp theme corpus, follow its adjacent `README.md`: review individual
theme provenance, update the category-count gate in the render crate's
`build.rs`, and verify both render tests and crate packaging.
