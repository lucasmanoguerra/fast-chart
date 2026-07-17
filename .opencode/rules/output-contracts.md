# Contratos de salida (handoffs parseables)

Al **cerrar un turno de fase**, el agent activo imprime **un bloque** al final.  
El orquestador no avanza la machine sin el bloque (o lo completa él si el agent falló el formato).

Convención: bloque en mayúsculas, campos `key: value`, listas con `-`.

---

## PO → `STORY_READY`

```
STORY_READY
hu: HU-001
path: spec/stories/hu-001.md
type: library|feature|api|cli|adapter|chore
priority: high|medium|low
open_questions:
  - …
next: contracts | wait_director
```

---

## Arqui → `CONTRACTS_READY`

```
CONTRACTS_READY
hu: HU-001
paths:
  - spec/contracts/….md
adr: null | spec/architecture/decisions/adr-….md
specialists_recommended:
  - ux-api
risks:
  - …
next: hard_spec
```

---

## SpecWriter → `SPEC_READY`

```
SPEC_READY
hu: HU-001
path: spec/features/hu-001.md
template_used: hard-spec-library
gherkin_scenarios: 8
open_questions:
  - …
ask: /approve | /reject
gate: hard_spec
```

---

## QArgento (plan) → `TEST_PLAN_READY`

```
TEST_PLAN_READY
hu: HU-001
path: spec/test-plans/hu-001.md
acceptance: N
unit: N
integration: N
ask: /approve | /reject
gate: test_plan
```

---

## QArgento (rojo) → `TESTS_RED`

```
TESTS_RED
hu: HU-001
commands:
  - cargo test …
failing: N
paths:
  - tests/…
next: green
```

---

## DevSenior → `IMPL_DONE`

```
IMPL_DONE
hu: HU-001
files_touched:
  - src/…
tests: red_to_green
commands:
  - cargo test …
notes_for_revisor:
  - …
next: review
```

---

## Revisor → `REVIEW_RESULT`

```
REVIEW_RESULT
hu: HU-001
status: pass | request_changes
findings:
  - severity: blocker|major|minor
    file: …
    issue: …
    suggestion: …
ask: /approve | fix_loop
gate: review   # si request_changes
```

---

## Architect-guard → `ARCHITECT_GUARD`

```
ARCHITECT_GUARD
status: pass | fail
findings:
  - severity: blocker|major|minor
    file: …
    issue: …
    suggestion: …
```

---

## QArgento (mutation) → `MUTATION_DONE`

```
MUTATION_DONE
hu: HU-001
score: 0.93
threshold: 0.90
report: mutation/report_hu-001.md
survivors: 0
equivalents: 0
next: docs | improve_tests
```

---

## Manteca → `DOCS_DONE`

```
DOCS_DONE
hu: HU-001
paths:
  - README.md
  - CHANGELOG.md
  - knowledge/…
commit: pending | hash
next: done_check
```

---

## Orquestador → `CONTEXT_PACK` / `PHASE_TRANSITION` / `QUALITY_SCORE`

```
PHASE_TRANSITION
from: hard_spec
to: test_plan
trigger: /approve
gate_cleared: hard_spec
machine: .opencode/state/machine.yaml
```

```
QUALITY_SCORE
hu: HU-001
path: spec/reviews/hu-001.md
architecture: 90
testing: 88
api: 85
docs: 80
perf_risk: 90
overall: 87
pass: true
```

---

## Spike → `SPIKE_DONE`

```
SPIKE_DONE
id: SPIKE-001
path: spec/spikes/…
recommendation: …
adr: null | path
unlocks_hu: […]
```

---

## Reglas

1. Un bloque de cierre por turno de agent (puede haber prosa antes).  
2. Paths siempre repo-relative.  
3. Si `ask: /approve`, **no** avanzar hasta el comando.  
4. El orquestador actualiza `machine.yaml` en `PHASE_TRANSITION`.
