# Convenciones — React

## Límites

- UI components ≠ domain logic: hooks de dominio testables sin DOM cuando se pueda.
- Estado: local primero; store global solo con motivo.
- Data fetching: capa clara (react-query/swr/fetchers), no fetch en medio del JSX sin orden.

## Componentes

- Un componente, una responsabilidad visual o de composición.
- Props tipadas; evitar prop drilling profundo (composition / context acotado).
- Accesibilidad: roles, labels, teclado (WCAG razonable).

## Testing

- React Testing Library: comportamiento, no implementación.
- E2E (Playwright/Cypress) para flujos críticos, no para todo.

## Performance

- Memoización con medición; no por defecto en todos lados.
- Code-split rutas pesadas.
