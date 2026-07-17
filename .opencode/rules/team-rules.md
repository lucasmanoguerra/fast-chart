# Reglas del equipo — Fábrica de software

> Versión 2.1 · Proceso genérico multi-lenguaje · Tono rioplatense  
> Context packs · state machine · output contracts · modes · DoD+score

## 1. Filosofía

- **Spec First (SDD):** sin Hard Spec aprobado no hay tests ni código de producción.
- **TDD:** tests primero (deben fallar), después implementación mínima.
- **SRP:** un archivo, una responsabilidad; un tipo/export público principal por archivo (salvo builders/errores del mismo agregado).
- **Mutation / calidad de tests:** umbral por defecto **≥ 90%** (o equivalente del lenguaje). Mutantes equivalentes documentados.
- **Desacoplamiento:** depender de abstracciones (traits/interfaces), no de infraestructura.
- **Documentación viviente:** README, architecture, specs, knowledge y changelog se actualizan con el cambio.
- **Fábrica intocable en features:** no modificar `.opencode/` salvo pedido explícito de mejorar el proceso.

## 2. Flujo de HU (inamovible)

Phases en `state/machine.yaml` (ver `state-machine.md`):

1. PO → story  
2. Arqui → contratos (+ ADR si hay trade-off)  
3. SpecWriter → Hard Spec → **approve** (`gate: hard_spec`)  
4. QArgento → plan → **approve** → tests rojos  
5. DevSenior → verde mínimo  
6. Revisor (+ architect-guard si aplica)  
7. Mutation / quality gate  
8. Manteca → docs/knowledge/commit  
9. `/done-hu` con DoD + **quality score**  

Cada fase cierra con **output contract** (`output-contracts.md`).  
Contexto por **context pack**, no catálogo entero.

## 3. Idioma y tono

- Comunicación del equipo: español rioplatense.
- Código e identificadores públicos: convención del lenguaje (casi siempre inglés).
- Commits: conventional commits (`feat`, `fix`, `docs`, `chore`, `refactor`, `test`, `perf`).

## 4. Ramas

- Feature: `feature/HU-###-descripcion-corta`
- No merge a main sin review + tests verdes + DoD

## 5. Herramientas (OpenCode)

- **engram:** memoria de decisiones/sesiones  
- **context7:** docs de libs bajo demanda  
- **codegraph:** acoplamiento, callers/callees, blast radius  
- **gh:** issues/PRs/repos  
- Tooling de lenguaje: ver `languages/*.md`

## 6. Penalizaciones (con humor)

- Romper SRP → refactor obligatorio antes de seguir.  
- Saltear mutation → el QA te frena.  
- Docs viejas → Manteca no cierra la HU.
