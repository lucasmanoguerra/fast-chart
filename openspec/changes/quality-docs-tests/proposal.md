# Proposal: quality-docs-tests

## Intent

Eliminate all compilation warnings, fix test-mode build failures, add crate-level documentation (README.md + api.md) in Spanish, improve test quality with Spanish comments, and establish a test classification system across all 12 crates.

## Scope

### In Scope
- Fix 116 clippy warnings to 0 (derive impls, unnecessary casts, unused imports, loops, etc.)
- Fix `serde_json` missing dev-dependency in `fc-primitives` and `fc-domain`
- Add README.md (Spanish) to each of the 12 crates
- Add api.md (Spanish) to each of the 12 crates
- Add detailed Spanish comments to all existing tests (fc-app, fc-renderer-wgpu)
- Create test suites for crates with 0 tests: fc-drawing, fc-render, fc-cache, fc-input, fc-animation, fc-sessions, fc-theme, fc-primitives, fc-domain, fc-examples
- Classify every test as deterministica (always pass) or fragil (may fail)
- Organize integration tests in `tests/` directories per crate where applicable

### Out of Scope
- New features or functionality changes
- Architecture restructuring
- Performance optimizations
- Cross-platform testing
- CI/CD pipeline changes

## Capabilities

### New Capabilities
- `crate-documentation`: README.md and api.md per crate in Spanish
- `test-quality`: Test comments in Spanish, test classification system
- `test-coverage`: Test suites for uncovered crates

### Modified Capabilities
- `code-quality`: Clippy zero-warning compliance (existing code quality work)

## Approach

**Phase 1 — Fix Compilation (dependency order: primitives → domain → rest)**
- Add missing `serde_json` dev-dependencies
- Apply `cargo clippy --fix` for mechanical fixes (derive impls, casts, unused imports)
- Manual fixes for complex warnings (too many arguments, loop patterns)

**Phase 2 — Test Comments (existing tests first)**
- Add Spanish comments to fc-app and fc-renderer-wgpu test files
- Classify each test as deterministica/fragil with rationale

**Phase 3 — New Test Suites (domain-driven order)**
- fc-primitives → fc-domain → fc-cache → fc-drawing → fc-input → fc-sessions → fc-animation → fc-render → fc-theme → fc-examples

**Phase 4 — Documentation (Spanish, per crate)**
- README.md: purpose, modules, quick-start, dependencies
- api.md: public API surface, traits, structs, functions

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `fc-primitives/Cargo.toml` | Modified | Add serde_json dev-dependency |
| `fc-domain/Cargo.toml` | Modified | Add serde_json dev-dependency |
| `fc-*/src/**/*.rs` | Modified | Clippy warning fixes |
| `fc-*/README.md` | New | Per-crate README in Spanish |
| `fc-*/api.md` | New | Per-crate API docs in Spanish |
| `fc-*/tests/*.rs` | New/Modified | New test files + comments on existing |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Clippy fix changes break behavior | Low | Run full test suite after each phase |
| New tests mask real bugs | Low | Classify tests and document edge cases |
| Scope creep on documentation | Medium | Strict per-crate template, review each batch |

## Rollback Plan

Each phase is independently revertible via `git checkout`. If a phase introduces regressions, revert that phase only. All changes are additive (new files + warnings fixes) — no behavior changes.

## Dependencies

- None external. Internal dependency: Phase 1 must complete before Phase 3 (tests need compilation to succeed).

## Success Criteria

- [ ] `cargo clippy --workspace --all-targets` produces 0 warnings
- [ ] `cargo test --workspace` compiles and passes in test mode
- [ ] 12 README.md files exist (one per crate), in Spanish
- [ ] 12 api.md files exist (one per crate), in Spanish
- [ ] All existing tests have detailed Spanish comments
- [ ] Every test is classified as deterministica or fragil
- [ ] fc-drawing has ≥1 test file
