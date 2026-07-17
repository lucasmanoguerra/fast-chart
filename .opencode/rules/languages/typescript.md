# Convenciones — TypeScript

## Proyecto

- `strict: true` en tsconfig.
- ESM o CJS según ecosistema; documentar dual publish si aplica.
- Package exports explícitos (`exports` map); no filtrar `src/` interno.

## Estilo

- Preferir `unknown` a `any`; acotar con type guards.
- Errores tipados (Result-like o error classes de dominio).
- Sin default export spaghetti; named exports estables en public API.

## Libs

- SemVer + changelog.
- Tests: vitest/jest + tipos (`tsc --noEmit`).
- Mutation opcional (Stryker).
- API surface revisada en breaking changes.

## Calidad

- ESLint + Prettier (o Biome).
- `publint` / `arethetypeswrong` en packages publicados.
