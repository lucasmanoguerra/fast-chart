# Clean Code & convenciones de código

## Archivos y SRP

- **Un archivo = una responsabilidad.**
- Preferir **un tipo/export público principal** por archivo.
- Cohesión OK: `Foo` + `FooBuilder` + `FooError` del mismo agregado.
- Mal: `Foo` + `Notifier` + `Parser` en el mismo file.

Pregunta del Revisor: *“¿Este archivo tiene una sola razón para cambiar?”*

## Naming

- Nombres que digan el **porqué del dominio**, no la tecnología.
- Evitar `Manager`, `Helper`, `Util`, `Common`, `Misc` sin dominio.
- Funciones: verbos; tipos: sustantivos; booleanos: `is_` / `has_` / `can_`.

## Funciones

- Cortas, un nivel de abstracción por bloque.
- Pocos parámetros; si hay muchos → struct de opciones/config.
- Sin side-effects ocultos: si muta o I/O, que el nombre/contrato lo diga.

## Errores

- Tipados en libs (`Result` + error enum / error type del lenguaje).
- No tragar errores; no panics en paths de librería.
- `unwrap`/`expect` solo en tests o en `main`/binarios justificados.
- Un estilo de error por crate/package (no mezclar 4 libs de error).

## Comentarios y docs

- Documentar **porqués** e invariantes, no lo obvio.
- API pública: docs generables (rustdoc / TSDoc / godoc / etc.).
- Comentarios TODO con HU o issue ref.

## Dependencias

- Preferir abstracciones inyectadas.
- Nuevas deps: justificación (ADR si es estructural).
- Evitar dependencias solo para 5 líneas.

## Complejidad

- Preferir claridad a micro-optimización prematura.
- Hot paths: medir antes (`/bench`, criterion, etc.).
- Concurrencia: documentar modelo de ownership/locks.

## Formato y lint

- Formatter + linter del stack en CI.
- Warnings del linter tratados como deuda; en libs preferir zero warnings públicos.
