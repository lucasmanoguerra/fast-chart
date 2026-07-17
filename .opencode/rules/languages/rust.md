# Convenciones — Rust (libs y bins)

## Toolchain

- `rust-toolchain.toml` pinneado cuando el proyecto es serio.
- Edition estable reciente; `rust-version` (MSRV) declarado en libs públicas.
- `cargo fmt` · `clippy -D warnings` (o allow list justificada) · tests · doc.

## Librerías

- Crate type library clara; examples como consumidores.
- `thiserror` (lib) vs `anyhow` (bins/examples).
- Sin `unwrap` en API de lib.
- `#![forbid(unsafe_code)]` por default; si hay `unsafe`, módulo aislado + SAFETY comments + tests.
- Features cargo documentadas; default features mínimas.
- SemVer: `cargo semver-checks` / public-api en cambios de superficie.
- `Cargo.lock` en apps; en libs puras suele ignorarse (team decide y documenta).

## Arquitectura Rust

- Traits en contratos; impls en adapters.
- Evitar `Arc<Mutex<T>>` en dominio sin justificación.
- Preferir ownership claro; documentar `Send`/`Sync` en tipos públicos.
- Módulos por dominio; `pub use` selectivo en `lib.rs`.

## Tests & calidad

- Unit + `tests/` integration.
- `cargo mutants` threshold 90%.
- Optional: nextest, criterion, cargo fuzz, deny/audit, machete/udeps.
- Benchmarks en `benches/` para hot paths.

## Async

- Elegir runtime con ADR si es dependencia pública.
- Libs async: preferir traits genéricos / `Future` sin forzar runtime cuando se pueda.
