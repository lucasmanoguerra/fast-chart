# Convenciones — Zig

## Estilo

- Allocator explícito: no esconder allocs.
- Errores con error sets claros; propagar con `try`.
- `comptime` con moderación y tests de compilación.

## Libs

- API mínima; documentar ownership de memoria en comentarios públicos.
- Build con `build.zig` reproducible.
- Tests en el lenguaje (`zig test`).

## Seguridad

- Cuidado con `@ptrCast` / undefined behavior; aislar y testear.
