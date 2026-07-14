# Contributing to Fast Chart

Thank you for your interest in contributing to **Fast Chart**.

Before you dive in, please read this guide fully.

## Table of Contents

- [Issue-First Workflow](#issue-first-workflow)
- [Development Setup](#development-setup)
- [Testing](#testing)
- [Commit Convention](#commit-convention)
- [Pull Request Rules](#pull-request-rules)
- [Code of Conduct](#code-of-conduct)

## Issue-First Workflow

**No PR without an issue. No exceptions.**

1. **Open an issue** using the appropriate template ([Bug Report](https://github.com/lucasmanoguerra/fast-chart/issues/new?template=bug_report.md) or [Feature Request](https://github.com/lucasmanoguerra/fast-chart/issues/new?template=feature_request.md))
2. **Wait for approval** — a maintainer will add the `status:approved` label when the issue is ready to be worked on
3. **Comment on the issue** to let others know you're working on it
4. **Open a PR** referencing the approved issue

PRs that are not linked to an approved issue will be **automatically rejected** by CI.

## Development Setup

### Prerequisites

- **Rust** 1.75 or later ([rustup](https://rustup.rs/))
- **Git**
- A GPU-capable system (for running the app)

### Setup

```bash
# Clone the repo
git clone https://github.com/lucasmanoguerra/fast-chart.git
cd fast-chart

# Build
cargo build

# Run tests
cargo test --workspace

# Run the demo app
cargo run --release -p fast-chart-app
```

## Testing

All contributions must include tests. We use Rust's built-in test framework.

```bash
# Run all tests
cargo test --workspace

# Run with output
cargo test --workspace -- --nocapture

# Run a specific test
cargo test -p fast-chart-domain test_bar_creation
```

### Test Guidelines

- **Unit tests** go in the same file as the code, inside `#[cfg(test)]` modules
- **Integration tests** go in `tests/` directories
- Tests must be deterministic — no time-dependent or random-dependent tests
- Use `#[test]` for normal tests, `#[bench]` for benchmarks

## Commit Convention

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

| Type | Description |
|------|-------------|
| `feat` | New feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `style` | Code style (formatting, missing semicolons, etc.) |
| `refactor` | Code refactoring without functional changes |
| `perf` | Performance improvement |
| `test` | Adding or updating tests |
| `chore` | Build process, CI, tooling |

### Scopes

| Scope | Description |
|-------|-------------|
| `domain` | fast-chart-domain |
| `core` | fast-chart-core |
| `app` | fast-chart-app |
| `render` | Rendering pipeline |

### Examples

```
feat(domain): add ADX indicator
fix(app): correct pane resize on high-DPI displays
docs(readme): update installation instructions
refactor(core): simplify viewport coordinate conversion
test(domain): add edge cases for Bollinger Bands
```

## Pull Request Rules

### Branch Naming

```
feat/<short-description>
fix/<short-description>
docs/<short-description>
refactor/<short-description>
```

### PR Checklist

Before submitting a PR, ensure:

- [ ] Code compiles without warnings: `cargo build`
- [ ] All tests pass: `cargo test --workspace`
- [ ] No clippy warnings: `cargo clippy --workspace`
- [ ] Code is formatted: `cargo fmt --all`
- [ ] New code has tests
- [ ] Documentation is updated if needed
- [ ] PR references an approved issue

### PR Size

- **Small PRs** (< 200 lines changed) are preferred
- **Large PRs** (> 400 lines) require `size:exception` label from a maintainer
- Split large changes into chained PRs when possible

### Review Process

1. A maintainer will review your PR within 72 hours
2. Address feedback in new commits (don't force-push during review)
3. Once approved, a maintainer will merge your PR

## Architecture Guidelines

Follow the hexagonal architecture rules:

- **Domain** (`fast-chart-domain`): Zero dependencies. Pure types and logic.
- **Core** (`fast-chart-core`): Depends only on Domain. Application logic.
- **App** (`fast-chart-app`): Depends on Core. Adapters (GPU, input, config).

### Adding a New Indicator

1. Create a new file in `fast-chart-domain/src/indicators/`
2. Implement the `Indicator` trait
3. Add tests in the same file
4. Register in `fast-chart-domain/src/indicators/mod.rs`
5. Add to the indicator list in `fast-chart-core/src/app/indicator_service.rs`

### Adding a New Renderer

1. Create a new file in `fast-chart-app/src/adapters/rendering/`
2. Implement the `ChartRenderer` trait
3. Register in `GpuRenderer`
4. Add tests

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Focus on constructive feedback
- Help newcomers learn
- Celebrate contributions

### Enforcement

Maintainers will enforce the Code of Conduct. Violations may result in temporary or permanent ban from the project.

## Questions?

Open a [Discussion](https://github.com/lucasmanoguerra/fast-chart/discussions) or reach out to the maintainer.
