# API Reference — fc-domain

## Tipos principales

### `Crosshair`
Crosshair del chart con modo imán. Constructor: `Crosshair::new(mode: MagnetMode)`.

### `MagnetMode`
Modo de imán: `Disabled`, `Nearest`, `Open`, `Close`, `High`, `Low`.

### `PriceScale`
Escala de precio con rango, modo y opciones. Constructor: `PriceScale::new(id, mode)`.

### `PriceScaleMode`
Modo de escala: `Normal`, `Percentage`, `Indexed`, `Logarithmic`.

### `PriceScaleOptions`
Opciones de escala de precio (auto-scale, invert, margins).

### `Viewport`
Rango visible del chart con time_start, time_end, value_min, value_max.

### `Marker`
Marcador visual en el chart. Constructor: `Marker::new(timestamp, price, shape, position)`.

### `MarkerShape`
Forma del marcador: `ArrowUp`, `ArrowDown`, `Circle`, `Square`, `Diamond`.

### `MarkerPosition`
Posición relativa: `AboveBar`, `BelowBar`, `InBar`.

### `PriceLine`
Línea de precio horizontal con estilo y etiqueta.

## Traits

### `Indicator`
Trait para indicadores financieros. Métodos: `name()`, `update()`, `values()`.

## Enums auxiliares

### `DrawingSet`
Conjunto de dibujos con operaciones de agregar, eliminar, seleccionar.

### `LabelPosition`
Posición de etiqueta: `Left`, `Right`, `Center`.

### `PriceLineId`
ID único para líneas de precio.

### `PriceFormatter`
Trait + `DefaultPriceFormatter` para formateo de precios.
