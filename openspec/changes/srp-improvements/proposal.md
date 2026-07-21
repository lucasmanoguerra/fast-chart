# Proposal: SRP Improvements in fast-chart

## Intent

Address Single Responsibility Principle violations in 4 components within `fc-app` and `fc-render` to improve maintainability, testability, and adherence to hexagonal architecture.

## Scope

### In Scope
- Extract `ViewportConstraints` from `ViewportManager` (low risk, first step)
- Extract `PaneState` from `Pane` (medium risk, high value)
- Split `ChartController` responsibilities (medium risk, requires careful boundary design)
- NOT touching `DrawingManager` in this change (deferred)

### Out of Scope
- `DrawingManager` refactoring (deferred to future change)
- Performance optimizations
- New features or API changes
- Changes to public API surface (all changes are internal)

## Capabilities

### New Capabilities
- `viewport-constraints`: Extracted viewport constraint logic from ViewportManager
- `pane-state`: Extracted pane state management from Pane struct

### Modified Capabilities
- `interaction`: ChartController split affects interaction handling boundaries
- `multi-pane-layout`: PaneState extraction changes internal pane structure

## Approach

### Phase 1: Extract ViewportConstraints (low risk)
- Create `ViewportConstraints` struct in `fc-app/src/app/viewport_constraints.rs`
- Move `min_zoom`, `max_zoom`, `min_visible_bars`, `max_visible_bars` fields
- Move `apply_zoom` and `apply_pan` methods that use these constraints
- Keep `ViewportManager` as a thin wrapper or rename to `ScaleFactory`
- Update tests to use new struct

### Phase 2: Extract PaneState (medium risk)
- Create `PaneState` struct grouping: series, indicators, drawings, markers, price_lines, formatter
- `Pane` retains: id, viewport, height, visible, price_scales, primary_scale_id, layers
- `Pane` gets `pub state: PaneState` field (or methods that delegate)
- Update all callers that access pane internals

### Phase 3: Split ChartController (medium risk)
- Extract `DataPollingService` handling `tick()` logic
- Extract `ViewportInteractionService` handling `handle_input()` viewport commands
- Keep `ChartController` as orchestrator that coordinates services
- Preserve existing test coverage

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `fc-app/src/app/viewport_management.rs` | Modified | Split into ViewportManager + ViewportConstraints |
| `fc-app/src/app/viewport_constraints.rs` | New | ViewportConstraints struct and impl |
| `fc-app/src/app/pane/mod.rs` | Modified | Pane delegates to PaneState |
| `fc-app/src/app/pane/state.rs` | New | PaneState struct |
| `fc-app/src/app/chart_controller.rs` | Modified | ChartController delegates to services |
| `fc-app/src/app/data_polling.rs` | New | DataPollingService |
| `fc-app/src/app/viewport_interaction.rs` | New | ViewportInteractionService |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Breaking existing tests | Medium | Run `cargo test` after each phase; fix compilation errors immediately |
| Introducing performance regression | Low | These are structural changes, not hot path modifications |
| Over-engineering the split | Medium | Keep interfaces minimal; avoid premature abstraction |
| Merge conflicts with parallel work | Low | Coordinate with other changes; keep PRs focused |

## Rollback Plan

- Each phase is independently revertible via `git revert`
- No database migrations or external dependencies to roll back
- Tests serve as safety net; if tests pass, behavior is preserved

## Dependencies

- No external dependencies
- Phase 2 depends on Phase 1 being stable
- Phase 3 is independent of Phase 2

## Success Criteria

- [ ] All existing tests pass after each phase
- [ ] `ViewportConstraints` has clear, single responsibility
- [ ] `PaneState` groups related state without breaking encapsapsulation
- [ ] `ChartController` responsibilities are clearly separated
- [ ] No public API changes (internal refactoring only)
- [ ] Code coverage maintained or improved