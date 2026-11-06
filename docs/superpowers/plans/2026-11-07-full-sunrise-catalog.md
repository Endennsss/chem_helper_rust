# Complete Sunrise Catalog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Publish v1.1.0 with all Sunrise chemical reactions available immediately and grouped by category.
**Architecture:** Complete importer + factual bundled snapshot + stable reaction selections and canonical manufacturing aliases; existing Tauri/Vue UI is extended.
**Tech Stack:** Rust, serde, reqwest, Tauri, Vue, Node tests, native GitHub Actions.
**Spec:** docs/superpowers/specs/2026-11-07-full-sunrise-catalog.md

## Global Constraints
- Exactly cover every audited reaction ID, including variants, effect-only and multi-output recipes.
- Same-commit data and Russian reagent localization; preserve favorites and last good cache.
- Source-only Git repository and Linux/Windows release.

## Review Focus
- Cycles and base-dispenser substances must not make the entire catalog unusable.
- Effect-only and multiple-product reactions must not invent an output quantity.
- Truncated API listings and failed files must not replace a full cache with a partial one.
- Old version caches and favorite IDs must survive upgrade without limiting catalog to 35/87.
- Category filtering and mixed batch/volume queues must stay usable and explicit.

### Task 1 — complete data model/import
- [x] Add failing synthetic tests for categories, variants, multiple/no outputs, inherited groups and machine conditions.
- [x] Implement importer and backward-compatible product aliases with a DAG for automatic prerequisites.
- [x] Generate factual snapshot from full sparse checkout; assert exact audited reaction ID coverage.

### Task 2 — refresh/cache and calculation
- [x] Test legacy cache replacement, complete coverage, file-list truncation and offline full catalog.
- [x] Implement same-commit recursive source discovery, parallel downloads and atomic complete cache replacement.
- [x] Test fractional outputs, batch quantities, catalysts and all bundled reactions.

### Task 3 — interface
- [x] Test category counts/filter and mixed quantity units.
- [x] Add category selector, source status, outputs, required equipment, special effects and batch labeling.
- [x] Run native GUI smoke, review full changes and fix important issues.

### Task 4 — release
- [x] Bump v1.1.0; document exact source coverage and supported behaviors.
- [ ] Run tests/clippy/builds, push, tag, verify release and checksums on native CI.
