# Hard Spec (CLI): {{título}}

> **ID**: {{id}} · **Versión**: {{version}} · **Estado**: {{estado}}

---

## 1. Resumen

Comando(s), audiencia (dev/ops/user), valor.

---

## 2. Gherkin

```gherkin
Feature: CLI {{cmd}}
  Scenario: usage feliz
    Given binario instalado
    When el usuario ejecuta `{{cmd}} {{args}}`
    Then exit code es 0
    And stdout contiene {{}}
```

---

## 3. Interfaz

| Elemento | Spec |
|----------|------|
| Bin name | |
| Subcommands | |
| Flags / env vars | |
| Config file | |
| Exit codes | 0 ok, 1 usage, 2 runtime, … |
| stdout / stderr policy | machine-readable? |

---

## 4. Errores y UX

Mensajes claros; no panics; `--help` completo.

---

## 5. Tests

- Unit de parsing  
- Integration con temp dirs  
- Snapshots de help opcional  
