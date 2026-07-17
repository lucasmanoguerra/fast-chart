# Convenciones — Python

## Estilo

- Type hints en API pública; `py.typed` en packages tipados.
- Errores con excepciones de dominio; evitar bare `except`.
- Packaging moderno (pyproject.toml / src layout).

## Calidad

- ruff/black o ruff format · mypy/pyright · pytest.
- Hypothesis para propiedades cuando aporte.
- Virtual env / lock según app vs lib.

## Libs

- SemVer; changelog; extras opcionales documentados.
- No side effects al importar el package.
