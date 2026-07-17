# Hard Spec (Adapter / Infrastructure): {{título}}

> **ID**: {{id}} · **Versión**: {{version}} · **Estado**: {{estado}}  
> **Puerto que implementa**: {{trait_o_interface}}

---

## 1. Resumen

Qué sistema externo o recurso adapta (DB, broker, GPU, FS, HTTP client, OS).

---

## 2. Gherkin

```gherkin
Feature: Adapter {{nombre}}
  Scenario: operación feliz
    Given {{recurso_disponible_o_fake}}
    When se invoca {{metodo_del_port}}
    Then el dominio recibe {{resultado}}

  Scenario: fallo del exterior
    Given {{recurso_caido}}
    When se invoca {{metodo}}
    Then se propaga error tipado {{variante}}
```

---

## 3. Port (contrato)

Link a `spec/contracts/…`. El adapter **no** ensancha el port sin ADR.

---

## 4. Detalles de infraestructura

| Tema | Decisión |
|------|----------|
| Tecnología | |
| Timeouts / retries | |
| Pooling | |
| Observability | |
| Feature flags | |

---

## 5. Límites de capa

- El dominio no importa este adapter.
- Wiring solo en composition root / host.

---

## 6. Tests

- Unit con fakes del otro lado del port  
- Integration con testcontainers / mocks de GPU / FS temp  
- Chaos: timeout, disconnect  

---

## 7. Seguridad

Credenciales, path traversal, untrusted input, sandbox.
