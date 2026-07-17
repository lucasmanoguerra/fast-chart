# Changelog — OpenCode Factory

Formato Keep a Changelog. Versionado SemVer de la **fábrica** (no del producto).

## [2.1.0] — 2026-07-16

### Added

- Context packs (`agents/context-packs.md`) con carga selectiva por fase/stack.
- State machine liviana (`state/machine.yaml` + `rules/state-machine.md`).
- Contratos de salida obligatorios en agents core (`rules/output-contracts.md`).
- Playbook duro `/done-hu` + quality score (`templates/quality-score.md`).
- Anti-patrones de fábrica (`rules/anti-patterns-factory.md`).
- Modos de operación (`rules/modes.md`): strict, standard, spike, hotfix, explore.
- `VERSION` y este changelog de fábrica.
- Comando `/mode` para cambiar modo.

### Changed

- `opencode.yaml` v2.1: packs, machine_file, mode default, rules de carga por capas.
- System prompt: packs, machine, modes, contratos de salida.
- `/approve` y `/reject` validan transiciones de la machine.
- Profiles core: bloque de output obligatorio al cerrar turno.

## [2.0.0] — 2026-07-16

### Added

- Rewrite genérico multi-lenguaje: rules, templates por caso de uso, agents matrix,
  multi/sub-agent, commands de largo plazo, sin perfil gui.
