# Warp themes

This directory contains the YAML theme definitions from
[`warpdotdev/themes`](https://github.com/warpdotdev/themes), pinned to the
commit recorded in `UPSTREAM_REVISION`.

Only YAML/YML theme definitions are vendored. Preview SVGs and background image
assets are intentionally excluded: Grok Build's `warp-sync` mode leaves the
terminal canvas transparent so Warp renders those itself, while pinned themes
use the YAML background color (or gradient midpoint).

The upstream repository is licensed under Apache-2.0; see `LICENSE`. Individual
theme files retain any more-specific provenance in their source comments. In
particular, `standard/cherry.yaml` and the three
`standard/soft_tactile_warp_*.yaml` files identify MIT-licensed source
material and their original authors. The product notices reproduce those MIT
attributions and terms for binary redistribution.

The upstream project also credits these theme sources used to bootstrap its
catalog: [iTerm Pencil](https://github.com/mattly/iterm-colors-pencil),
[Alacritty themes](https://github.com/alacritty/alacritty-theme),
[Base16 Alacritty](https://github.com/aarowill/base16-alacritty),
[Base16](https://github.com/chriskempson/base16),
[Solarized](https://ethanschoonover.com/solarized/),
[Dracula](https://draculatheme.com/), and
[Gruvbox](https://github.com/morhetz/gruvbox). Base16 themes were sourced from
the Alacritty collection maintained by `aarowill`; standard themes were
sourced from the Alacritty collection maintained by `eendroroy`.

## Refreshing the catalog

1. Audit and pin a new upstream commit in `UPSTREAM_REVISION` and the adjacent
   crate `build.rs` revision gate.
2. Replace only YAML/YML files under `base16/`, `standard/`,
   `special_edition/`, `stradicat/`, and `warp_bundled/` plus the upstream
   `LICENSE`; do not copy previews or image assets. Preserve per-theme source
   comments and review whether new files add more-specific license terms.
3. Update the expected category counts in the adjacent crate `build.rs` if the
   audited catalog size changed.
4. Run the render crate catalog and package tests, then update
   `THIRD-PARTY-NOTICES`, `third_party/NOTICE`, and `UPSTREAM_VERSIONS.md` with
   the new revision and any new provenance.
