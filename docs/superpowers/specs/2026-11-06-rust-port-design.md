# Chem Helper Rust

Authorized scope: independent Rust + Tauri desktop port in the ignored nested
`chem-helper-rust/` directory, pushed to Endennsss/chem_helper_rust with a Linux
and Windows release. Keep the Vue interface and original non-commercial license.

Use a Rust core crate for recipe parsing, localization, dependency expansion,
calculation and file persistence. Tauri invokes that crate directly: no Python,
local HTTP server or bundled Chromium. Preserve queue, favorites, search,
temperatures, tooltips and appearance settings. Open external links through the
Tauri opener. Persist user state atomically in app data; recover a damaged file
without silently overwriting it. Recipes use stable reagent IDs internally.

Fix the existing diamond-dependency undercount and rounding before summing.
Validate finite positive quantities, detect cycles, show catalysts separately
without consuming them. Parse SS14 tagged YAML and keep last valid cached data
when online refresh fails. Provide original supplementary recipes offline; fetch
upstream content at runtime rather than vendor third-party resources. Expose
refresh and data status in Settings, without blocking startup on network.

Keep dependencies, caches and release binaries out of Git. Use GitHub Actions
with native Linux and Windows runners to build/test and publish release assets.
A web-hosted edition is outside this separate desktop port's scope.
