# Convenciones — JavaScript

## Cuándo JS vs TS

- Libs nuevas: preferir TypeScript.
- JS legacy: JSDoc tipado donde aporte; migrar por módulos calientes.

## Estilo

- Strict mode; evitar globals.
- Errores con códigos/mensajes estables para consumidores.
- Public API pequeña; documentar side effects.

## Tooling

- Node LTS pinneado (`.nvmrc` / `engines`).
- Test runner del monorepo; coverage en paths críticos.
- Lint + format en CI.
