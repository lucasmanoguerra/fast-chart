# Hard Spec (Feature de producto/app): {{título}}

> **ID**: {{id}} · **Versión**: {{version}} · **Estado**: {{estado}}  
> **Fecha**: {{fecha}} · **Aprueba**: Director

---

## 1. Resumen ejecutivo

{{resumen}}

### Propósito / valor de usuario
{{proposito}}

### Alcance
{{alcance}}

### Fuera de alcance
{{fuera}}

### Actores
{{actores}}

---

## 2. Criterios de aceptación (Gherkin)

```gherkin
Feature: {{nombre}}

  Scenario: {{feliz}}
    Given {{pre}}
    When {{accion}}
    Then {{ui_o_efecto_observable}}

  Scenario: {{error}}
    Given {{pre}}
    When {{accion}}
    Then {{mensaje_o_estado_error}}

  Scenario: {{borde}}
    Given {{pre}}
    When {{accion}}
    Then {{resultado}}
```

---

## 3. UX / flujo (si aplica)

- Entry points: {{}}
- Estados de UI: loading / empty / error / success
- Accesibilidad mínima: {{}}

---

## 4. Contratos internos

Traits/interfaces y módulos tocados (link a `spec/contracts/`).

---

## 5. Datos y persistencia

- Modelos: {{}}
- Migraciones: {{}}
- Privacidad: {{}}

---

## 6. Efectos secundarios

Eventos, mails, jobs, analytics, caches.

---

## 7. Seguridad

Authz, validación de input, secretos, rate limits.

---

## 8. Estrategia de prueba

Unit / integration / e2e según riesgo.

---

## 9. Riesgos abiertos

{{riesgos}}
