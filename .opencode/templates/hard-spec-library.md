# Hard Spec (Library): {{título}}

> **ID**: {{id}} · **Versión**: {{version}} · **Estado**: {{estado}}  
> **Fecha**: {{fecha}} · **Stack**: {{lenguaje}} · **Aprueba**: Director

---

## 1. Resumen ejecutivo

{{resumen}}

### Propósito
{{proposito}}

### Alcance
{{alcance}}

### Fuera de alcance
{{fuera_alcance}}

### Consumidor de la API
Quién importa el package/crate y cómo (app host, otra lib, FFI, …).

---

## 2. Criterios de aceptación (Gherkin)

```gherkin
Feature: {{nombre}}

  Background:
    Given {{contexto}}

  Scenario: {{feliz_1}}
    Given {{pre}}
    When {{accion}}
    Then {{resultado}}

  Scenario: {{error_1}}
    Given {{pre}}
    When {{accion_invalida}}
    Then {{error_observable}}

  Scenario: {{borde_1}}
    Given {{pre_extrema}}
    When {{accion}}
    Then {{resultado}}
```

---

## 3. Contrato de API pública

### Módulo / package path
`{{path_publico}}`

### Tipos exportados

```
// Pseudocódigo o sintaxis del lenguaje
{{tipos}}
```

### Funciones / métodos / traits

| Símbolo | Firma resumida | Errores | Notas de semver |
|---------|----------------|---------|-----------------|
| {{sym}} | {{sig}} | {{err}} | {{semver}} |

### Errores

| Código / variante | Condición | Recoverable |
|-------------------|-----------|-------------|
| {{e}} | {{cond}} | sí/no |

### Invariantes
- {{inv_1}}
- Thread-safety: {{send_sync_o_equiv}}
- Panic/abort policy: {{policy}}

---

## 4. Dependencias

| Dependencia | Tipo | Trait/Interface | Notas |
|-------------|------|-----------------|-------|
| {{dep}} | dominio/adapter | {{iface}} | |

### Prohibiciones de capa
Ej.: dominio no depende de {{framework}}.

---

## 5. Efectos secundarios

- I/O: {{io}}
- Logs: {{logs}}
- Estado global: {{global}} (preferir ninguno)
- FS / red / GPU: {{}}

---

## 6. Performance & recursos

| Métrica | Target | Cómo se mide |
|---------|--------|--------------|
| {{m}} | {{t}} | bench/test |

Allocs / locks / clones esperados en hot path: {{notas}}

---

## 7. Estrategia de prueba

| Nivel | Qué | Path |
|-------|-----|------|
| Unit | | |
| Integration | | |
| Acceptance | escenarios Gherkin | |
| Mutation/fuzz | | |
| Bench | | opcional |

---

## 8. Documentación requerida al cerrar

- [ ] Docs de API pública  
- [ ] Example mínimo  
- [ ] Changelog si cambia superficie  
- [ ] ADR si hubo trade-off  

---

## 9. Riesgos y preguntas abiertas

{{riesgos}}
