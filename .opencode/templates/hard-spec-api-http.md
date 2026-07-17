# Hard Spec (HTTP/API): {{título}}

> **ID**: {{id}} · **Versión**: {{version}} · **Estado**: {{estado}}

---

## 1. Resumen

{{resumen}}

---

## 2. Gherkin

```gherkin
Feature: {{api_feature}}
  Scenario: {{feliz}}
    Given {{auth_y_pre}}
    When el cliente {{metodo}} {{path}}
    Then la respuesta es {{status}}
    And el body cumple el schema
```

---

## 3. Endpoints

### `{{METHOD}} {{path}}`

**Request**

| Campo | Tipo | Req | Validación |
|-------|------|-----|------------|
| | | | |

**Responses**

| Status | Significado | Body |
|--------|-------------|------|
| 200 | | |
| 400 | | |
| 401 | | |
| 404 | | |
| 429 | | |
| 500 | | |

**Headers:** Authorization, Idempotency-Key, …  

**Idempotencia / paginación / versionado:** {{}}

---

## 4. Errores de dominio (mapeo a HTTP)

| Error dominio | HTTP | code |
|---------------|------|------|
| | | |

---

## 5. Seguridad

Authn/z, CORS, rate limit, validación, PII.

---

## 6. Contratos de código

Handlers → use-cases → ports (link contracts).

---

## 7. Tests

Contract tests, integration con server de test, e2e críticos.
