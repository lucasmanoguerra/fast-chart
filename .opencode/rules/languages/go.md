# Convenciones — Go

## Layout

- `internal/` para no-exportable.
- Packages por dominio; evitar `util`/`common` gordos.
- `go.mod` con versiones claras.

## Estilo

- Errores con `fmt.Errorf("%w")` / tipos de error de dominio.
- Context en fronteras de API (`context.Context` primer param).
- Interfaces chicas del lado del consumidor.

## Calidad

- `gofmt` / `go vet` / `staticcheck` / tests race cuando haya concurrencia.
- Table-driven tests.
- Fuzz nativo en parsers.
- SemVer de modules (`/v2` path cuando aplique).
