# Interaction Specification

## Purpose

Define user interaction handling: pan, zoom, crosshair tracking, timeframe switching, and pane resizing. All input coordinates translate from screen space to domain space via Scale.

## Requirements

### Requirement: InteractionHandler port trait

An `InteractionHandler` trait MUST define methods for all input events: `on_mouse_move`, `on_mouse_drag`, `on_scroll`, `on_key_down`, and `on_resize_drag`. Each method SHALL return an `InteractionCommand` to the controller.

#### Scenario: Mouse move returns crosshair command

- GIVEN mouse position (400, 250) in screen space
- WHEN `on_mouse_move` is called
- THEN it returns `InteractionCommand::UpdateCrosshair` with the cursor position

#### Scenario: Unrecognized event

- GIVEN a keyboard event for an unbound key
- WHEN `on_key_down` is called
- THEN it returns `InteractionCommand::None`

### Requirement: Pan (click-drag)

Click-drag on the chart SHALL translate the viewport in time. Mouse movement SHALL proportionally shift the visible time range.

#### Scenario: Horizontal pan

- GIVEN a viewport covering 6.5 hours (9:30–16:00)
- WHEN the user drags left by 200 pixels at scale 800px per 6.5h
- THEN the viewport shifts forward by ~1.625 hours

#### Scenario: Pan outside data range

- GIVEN data from 9:30–16:00
- WHEN the user pans past 16:00
- THEN the viewport SHALL clamp to the last available data boundary

### Requirement: Zoom (scroll wheel)

Scroll wheel SHALL zoom the viewport around the cursor position. Scrolling up SHALL zoom in (shorter time range), scrolling down SHALL zoom out.

#### Scenario: Zoom in around cursor

- GIVEN viewport 9:30–16:00, cursor at 12:00
- WHEN the user scrolls up by one notch
- THEN the visible time range decreases, centered on 12:00

#### Scenario: Zoom out beyond data

- GIVEN data limited to 9:30–16:00
- WHEN zooming out would expand the viewport beyond data bounds
- THEN the viewport SHALL clamp to available data extent

### Requirement: Crosshair (mouse move)

Mouse movement without dragging SHALL show a crosshair: a vertical line at cursor x, a horizontal line at cursor y, and an OHLC tooltip.

#### Scenario: Crosshair follows cursor

- GIVEN the mouse at screen position (400, 250)
- WHEN on_mouse_move fires
- THEN a vertical line at x=400 and horizontal line at y=250 are rendered

#### Scenario: Crosshair tooltip shows OHLC

- GIVEN the nearest bar to cursor x is OHLC (100, 105, 99, 102)
- WHEN the crosshair tooltip renders
- THEN it displays formatted OHLC values with the bar's timestamp

### Requirement: Timeframe switching

Keyboard shortcuts SHALL switch the visible timeframe: `1` (1min), `5` (5min), `15` (15min), `60` (1hour), `D` (daily), `W` (weekly).

#### Scenario: Timeframe shortcut triggers aggregation

- GIVEN current view at 1-minute bars
- WHEN the user presses `5`
- THEN the view switches to 5-minute aggregated bars

#### Scenario: Invalid timeframe key

- GIVEN the chart in focus
- WHEN the user presses `3`
- THEN the chart ignores the key (no binding for 3-minute)

### Requirement: Resize pane divider

Dragging a pane divider SHALL resize adjacent panes. The interaction handler SHALL return `InteractionCommand::ResizePane` with the delta.

#### Scenario: Divider drag produces resize command

- GIVEN a divider between pane 0 and pane 1 at y=540
- WHEN dragged to y=480
- THEN a `ResizePane` command with delta −60px is returned

#### Scenario: Minimum height enforced

- GIVEN bottom pane at 60px minimum
- WHEN divider is dragged past the minimum boundary
- THEN the resize command SHALL clamp to the minimum valid height

### Requirement: Screen-to-domain coordinate translation

All interaction coordinates SHALL be translated from screen space to domain space via the `Scale` mapping.

#### Scenario: Screen x to timestamp

- GIVEN screen x=400, canvas width=800, time range 9:30–16:00
- WHEN translating to domain
- THEN the result is 12:45

#### Scenario: Screen y to price

- GIVEN screen y=250, canvas height=500, price range 100–110
- WHEN translating to domain
- THEN the result is 105.0
