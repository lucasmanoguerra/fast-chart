# Estándares para librerías de larga vida

Aplica a crates, packages npm, módulos Go, libs Zig/Python, etc.

## Producto = la librería

- El repo transmite cómo trabajar: CONTRIBUTING, architecture, examples, changelog, security.
- Pensá 5–10 años: cada dependencia y cada `pub` es deuda potencial.

## Public API

1. Diseñar traits/interfaces y tipos estables **antes** de implementar.
2. Superficie mínima; versionar con SemVer estricto.
3. Documentar breaking changes.
4. Ejemplos compilables en `examples/` (o equivalente).
5. Comando `/public-api` en cambios de superficie.

## Estructura recomendada (crece con el tiempo)

```
src/ or crates/ or packages/
examples/
tests/
benches/
fuzz/                 # si parsea input no confiable
docs/ or architecture/
knowledge/            # patrones, glosario, anti-patterns del proyecto
spec/                 # stories, features, contracts
.github/              # CI, issue/PR templates
```

## Domain first

- Carpetas por dominio, no `utils/helpers`.
- README por módulo importante.
- Dominio sin frameworks.

## Calidad de release

- Formatter, linter, tests, docs, audit de deps.
- `publish --dry-run` / pack dry-run antes de release.
- Benchmarks en cambios de hot path.
- Cargo/npm/go deny-audit según ecosistema.

## No es una app

- No meter UI framework “porque sí”.
- Binarios/examples son consumidores, no el core.
- Config y logging host-friendly (la lib no impone un logger global rígido).

## Knowledge del proyecto (no de la fábrica)

Patrones aprendidos → `knowledge/` del repo (ver `knowledge.md`).  
Nunca copiar reglas de producto dentro de `.opencode/`.
