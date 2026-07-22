# Design: quality-docs-tests

## Technical Approach

Four-phase incremental change. Phase 1 (clippy) unblocks everything. Phases 2-4 are independent after Phase 1. All documentation in Spanish. Test comments in Spanish. Each crate gets standardized README.md + api.md.

```
Phase 1: Fix Compilation ──→ Phase 3: New Test Suites
                                   ↑
Phase 2: Test Comments ────────────┤ (parallel with Phase 4)
Phase 4: Documentation ───────────┘
```

## Architecture Decisions

### Decision: Documentation language

**Choice**: Spanish (matching user preference)
**Alternatives considered**: English (standard for Rust crates)
**Rationale**: User explicitly requested Spanish documentation. Public API docs (doc comments) stay in English per Rust convention; README.md and api.md are Spanish.

### Decision: Test file organization

**Choice**: Inline `#[cfg(test)] mod tests` for unit-style tests; `tests/` dir for integration tests
**Alternatives considered**: All integration tests in `tests/`; all inline
**Rationale**: Follows existing patterns. fc-app already uses `tests/`, fc-theme/fc-sessions use inline. New crates follow the pattern that matches their coupling level.

### Decision: Test classification system

**Choice**: Doc comment header per test: `// Clasificación: determinística | frágil` + rationale
**Alternatives considered**: Attribute macro; separate metadata file
**Rationale**: Zero-cost, grep-friendly, visible in code review. Macro adds complexity for no benefit.

### Decision: README.md template structure

**Choice**: 7-section Spanish template per crate
**Alternatives considered**: Minimal (3-section); full RFC-style
**Rationale**: Balances completeness with maintainability. Sections: Nombre, Descripción, Módulos, Uso rápido, Dependencias, Tests, Licencia.

### Decision: api.md template structure

**Choice**: Module-by-module listing of public items (traits, structs, enums, functions)
**Alternatives considered**: Auto-generated from `cargo doc`; prose-only
**Rationale**: Auto-generated docs exist via `cargo doc`; api.md serves as a human-curated quick reference. Prose-only loses discoverability.

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `fc-*/README.md` (×12) | Create | Per-crate README in Spanish |
| `fc-*/api.md` (×12) | Create | Per-crate API reference in Spanish |
| `fc-primitives/Cargo.toml` | Modify | Add `serde_json` dev-dependency |
| `fc-domain/Cargo.toml` | Modify | Add `serde_json` dev-dependency |
| `fc-*/src/**/*.rs` | Modify | Clippy warning fixes (~116 warnings) |
| `fc-drawing/src/lib.rs` | Modify | Convert `//` comments to `//!` doc comments |
| `fc-cache/src/lib.rs` | Modify | Add `//!` crate doc comment |
| `fc-renderer-wgpu/src/lib.rs` | Modify | Add `//!` crate doc comment |
| `fc-drawing/tests/basic_tests.rs` | Create | First test file for fc-drawing |
| `fc-cache/tests/cache_tests.rs` | Create | Cache hit/miss/eviction tests |
| `fc-render/tests/coordinate_tests.rs` | Create | Coordinate pipeline tests |
| `fc-app/tests/*.rs` (×4) | Modify | Add Spanish comments + classification |
| `fc-renderer-wgpu/tests/*.rs` (×1) | Modify | Add Spanish comments + classification |

## Interfaces / Contracts

### README.md template (Spanish)

```markdown
# `fc-{name}`
> {una-línea-descripción}

## Descripción
{propósito del crate}

## Módulos
| Módulo | Descripción |
|--------|-------------|
| `mod_name` | {qué hace} |

## Uso rápido
```rust
// Ejemplo mínimo
```

## Dependencias
| Crate | Propósito |
|-------|-----------|
| `dep` | {para qué se usa} |

## Tests
Ejecutar: `cargo test -p fc-{name}`

## Licencia
{MIT | MIT/Apache-2.0}
```

### api.md template (Spanish)

```markdown
# API Reference — `fc-{name}`

## Módulo: `mod_name`
| Elemento | Tipo | Descripción |
|----------|------|-------------|
| `ItemName` | `struct/enum/trait/fn` | {qué hace} |
```

### Test classification format

```rust
/// Clasificación: determinística
/// Razonamiento: Solo depende de aritmética fija, sin I/O ni tiempo.
#[test]
fn bar_roundtrip() { ... }

/// Clasificación: frágil
/// Razonamiento: Depende de tiempo del sistema para verificar timeouts.
#[test]
fn session_timeout() { ... }
```

## Testing Strategy

| Crate | Test Type | Approach |
|-------|-----------|----------|
| fc-primitives | Unit (inline) | Bar creation, TimeSeries push/latest, Scale conversion |
| fc-domain | Unit (inline) | Crosshair modes, PriceScale formatting, DrawingSet CRUD |
| fc-cache | Unit + Integration | Cache hit/miss, LRU eviction, TTL expiration |
| fc-drawing | Integration (`tests/`) | Trait dispatch, AABB hit test, bounds combine |
| fc-input | Unit (inline) | Event constructors, ModifierFlags, KeyCode dedup |
| fc-sessions | Unit (inline) | Session duration, overnight logic, renderer output |
| fc-animation | Unit (inline) | Easing curves, AnimatedValue lifecycle, Engine GC |
| fc-render | Integration (`tests/`) | Pipeline frame execution, dirty tracking, coordinate transform |
| fc-theme | Unit (inline) | Builder, hot-swap, ThemeHandle clone, all tokens round-trip |
| fc-renderer-wgpu | Integration (`tests/`) | Pipeline + cache + scissor + vertex gen (existing, add comments) |
| fc-app | Integration (`tests/`) | Full pipeline with mocks (existing 4 files, add comments) |
| fc-examples | N/A | Binary crate, no unit tests |

## Threat Matrix

N/A — no routing, shell, subprocess, VCS/PR automation, executable-file classification, or process-integration boundary.

## Migration / Rollout

No migration required. All changes are additive (new files, doc comments, test files) plus mechanical clippy fixes. Each phase is independently revertible.

## Open Questions

- [ ] Should fc-examples get a README even though it's a binary crate with no lib.rs? (Recommendation: yes, with usage examples instead of module listing)
- [ ] Should api.md include re-exports or only locally-defined items? (Recommendation: include re-exports with `(re-export from fc-domain)` annotation)
