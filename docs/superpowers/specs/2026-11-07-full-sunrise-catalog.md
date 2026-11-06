# Complete Sunrise chemical catalog

User authorizes all chemical reaction prototypes (not lathe/microwave/item crafting),
categories, native Linux/Windows builds and a new release in chem_helper_rust.
Audit commit 87c75a47e4f9ef9eb6d8fc6a8f759a1b3fba900c: all 3806 prototype YAML
files scanned; 386 distinct type:reaction IDs in 19 source files, 487 reagents.
66 reactions have no reagent product and 31 have multiple reagent products.
The _Sunrise/Chemistry/Recipes directory is part of the scope.

Ship a compact factual normalized snapshot so clean offline startup has every
reaction. Discover online files recursively, honor non-truncated tree coverage,
load current consistent commit and cache only complete validated replacements.
Legacy 35/87 caches must not hide the bundled snapshot after upgrading.

Each reaction has its own stable reaction:ID selection, category, source path,
products, input quantities/catalysts, temperatures, mixer requirements and effects.
Keep every alternative for a product; canonical product aliases remain for old
favorites and automatic dependency plans. Automatic expansion uses an acyclic
preferred manufacturing graph, with supplied/irreducible reagents left as base
inputs, never dropping an otherwise valid cyclic/reversible reaction from catalog.

Single/multi-product quantities mean total reagent output volume. Effect-only
reactions use batches, and quantized reactions use whole batches. Show every
output and relevant conditions. Preserve fractional amounts rather than integer
rounding. Use Russian category labels, category counts/filter and ID-aware search.

Do not commit source checkout, dependencies, build output or release binaries.
Retain source provenance/commit and an audited ID manifest alongside compact data.
