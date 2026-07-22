# Checklist de Tests — fast-chart

## Resumen

| Métrica | Valor |
|---------|-------|
| **Total de tests** | 1395 |
| **Determinísticos (siempre pasan)** | 1394 |
| **Frágiles (pueden fallar)** | 1 |
| **Exitosos (última ejecución)** | 1395 (100%) |
| **Fallidos** | 0 |

### Desglose por Tipo

| Tipo | Cantidad |
|------|----------|
| Unit tests | 880 |
| Integration tests | 490 |
| Doc-tests | 25 |

---

## Tests por Crate

### fc-animation (24 tests)

**Determinísticos:** 24 | **Frágiles:** 0

| # | Test | Clasificación |
|---|------|---------------|
| 1 | `tests::animated_value_complete_at_end` | determinística |
| 2 | `tests::animated_value_complete_force` | determinística |
| 3 | `tests::animated_value_is_complete_flag` | determinística |
| 4 | `tests::animated_value_new` | determinística |
| 5 | `tests::animated_value_new_zero_duration_is_complete` | determinística |
| 6 | `tests::animated_value_retarget` | determinística |
| 7 | `tests::animated_value_update` | determinística |
| 8 | `tests::animated_value_update_past_end_caps` | determinística |
| 9 | `tests::animation_engine_active_count` | determinística |
| 10 | `tests::animation_engine_add_and_get` | determinística |
| 11 | `tests::animation_engine_gc` | determinística |
| 12 | `tests::animation_engine_remove` | determinística |
| 13 | `tests::animation_engine_replace_same_name` | determinística |
| 14 | `tests::animation_engine_update_all` | determinística |
| 15 | `tests::easing_clamps_at_boundaries` | determinística |
| 16 | `tests::easing_ease_in` | determinística |
| 17 | `tests::easing_ease_in_out_first_half` | determinística |
| 18 | `tests::easing_ease_in_out_second_half` | determinística |
| 19 | `tests::easing_ease_out` | determinística |
| 20 | `tests::easing_linear` | determinística |
| 21 | `tests::easing_spring_converges` | determinística |
| 22 | `fc-animation/src/lib.rs - AnimatedValue (line 82)` | determinística |
| 23 | `fc-animation/src/lib.rs - AnimationEngine (line 189)` | determinística |
| 24 | `fc-animation/src/lib.rs - apply_easing (line 282)` | determinística |

---

### fc-app (301 tests)

**Determinísticos:** 301 | **Frágiles:** 0

| # | Test | Clasificación |
|---|------|---------------|
| 25 | `app::chart_controller::tests::add_and_clear_indicator_overlays` | determinística |
| 26 | `app::chart_controller::tests::data_provider_start_stop` | determinística |
| 27 | `app::chart_controller::tests::handle_input_deactivates_crosshair` | determinística |
| 28 | `app::chart_controller::tests::handle_input_sets_crosshair` | determinística |
| 29 | `app::chart_controller::tests::handle_input_updates_viewport` | determinística |
| 30 | `app::chart_controller::tests::tick_processes_bar_events` | determinística |
| 31 | `app::chart_controller::tests::tick_processes_multiple_events` | determinística |
| 32 | `app::chart_controller::tests::tick_renders_when_dirty` | determinística |
| 33 | `app::chart_controller::tests::tick_skips_render_when_no_data` | determinística |
| 34 | `app::data_polling::tests::poll_marks_dirty` | determinística |
| 35 | `app::data_polling::tests::poll_no_data_no_dirty` | determinística |
| 36 | `app::data_polling::tests::poll_processes_bar_closed` | determinística |
| 37 | `app::data_polling::tests::poll_processes_multiple_events` | determinística |
| 38 | `app::data_polling::tests::start_stop_delegates_to_provider` | determinística |
| 39 | `app::frame_counter::tests::frame_counter_new` | determinística |
| 40 | `app::frame_counter::tests::frame_counter_tick_returns_none_within_second` | determinística |
| 41 | `app::indicator_service::tests::calculate_all` | determinística |
| 42 | `app::indicator_service::tests::calculate_all_empty_registry` | determinística |
| 43 | `app::indicator_service::tests::default_is_empty` | determinística |
| 44 | `app::indicator_service::tests::empty_registry` | determinística |
| 45 | `app::indicator_service::tests::get_nonexistent_returns_none` | determinística |
| 46 | `app::indicator_service::tests::names_returns_all` | determinística |
| 47 | `app::indicator_service::tests::register_and_get` | determinística |
| 48 | `app::indicator_service::tests::register_overwrites_existing` | determinística |
| 49 | `app::indicator_service::tests::register_zero_indicator_and_calculate` | determinística |
| 50 | `app::indicator_service::tests::remove_existing` | determinística |
| 51 | `app::indicator_service::tests::remove_nonexistent` | determinística |
| 52 | `app::layout::tests::grid_2x2` | determinística |
| 53 | `app::layout::tests::grid_2x2_with_gaps` | determinística |
| 54 | `app::layout::tests::grid_3x1` | determinística |
| 55 | `app::layout::tests::grid_fewer_panes_than_cells` | determinística |
| 56 | `app::layout::tests::grid_more_panes_than_cells` | determinística |
| 57 | `app::layout::tests::grid_pane_count_hint` | determinística |
| 58 | `app::layout::tests::grid_zero` | determinística |
| 59 | `app::layout::tests::horizontal_split_empty` | determinística |
| 60 | `app::layout::tests::horizontal_split_equal` | determinística |
| 61 | `app::layout::tests::horizontal_split_proportional` | determinística |
| 62 | `app::layout::tests::horizontal_split_with_gap` | determinística |
| 63 | `app::layout::tests::vertical_stack_default` | determinística |
| 64 | `app::layout::tests::vertical_stack_empty` | determinística |
| 65 | `app::layout::tests::vertical_stack_equal` | determinística |
| 66 | `app::layout::tests::vertical_stack_proportional` | determinística |
| 67 | `app::layout::tests::vertical_stack_with_gap` | determinística |
| 68 | `app::layout_manager::tests::add_pane_rebalances` | determinística |
| 69 | `app::layout_manager::tests::cannot_remove_last_pane` | determinística |
| 70 | `app::layout_manager::tests::compute_rects_uses_engine` | determinística |
| 71 | `app::layout_manager::tests::compute_rects_with_three_panes` | determinística |
| 72 | `app::layout_manager::tests::default_layout` | determinística |
| 73 | `app::layout_manager::tests::default_pane_heights` | determinística |
| 74 | `app::layout_manager::tests::divider_hit_test` | determinística |
| 75 | `app::layout_manager::tests::divider_hit_test_delegates_to_pane_divider` | determinística |
| 76 | `app::layout_manager::tests::divider_position_updates_on_drag` | determinística |
| 77 | `app::layout_manager::tests::drag_adjusts_heights` | determinística |
| 78 | `app::layout_manager::tests::engine_accessor_returns_immutable_reference` | determinística |
| 79 | `app::layout_manager::tests::hit_test_returns_correct_index` | determinística |
| 80 | `app::layout_manager::tests::min_height_enforced` | determinística |
| 81 | `app::layout_manager::tests::min_height_enforced_drag_down` | determinística |
| 82 | `app::layout_manager::tests::pane_pixel_dimensions` | determinística |
| 83 | `app::layout_manager::tests::pane_y_offset` | determinística |
| 84 | `app::layout_manager::tests::rebuild_dividers_after_add` | determinística |
| 85 | `app::layout_manager::tests::remove_pane_rebalances` | determinística |
| 86 | `app::layout_manager::tests::remove_pane_reindexes` | determinística |
| 87 | `app::layout_manager::tests::set_engine_swaps_layout_strategy` | determinística |
| 88 | `app::layout_manager::tests::sync_time_range` | determinística |
| 89 | `app::layout_manager::tests::sync_zoom` | determinística |
| 90 | `app::layout_manager::tests::three_pane_layout` | determinística |
| 91 | `app::layout_manager::tests::vertical_stack_compute_rects_matches_inline` | determinística |
| 92 | `app::pane::content::tests::pane_content_default_is_empty` | determinística |
| 93 | `app::pane::content::tests::pane_content_formatter_produces_output` | determinística |
| 94 | `app::pane::content::tests::pane_content_push_indicator` | determinística |
| 95 | `app::pane::content::tests::pane_content_push_series` | determinística |
| 96 | `app::pane::content::tests::pane_delegates_to_content` | determinística |
| 97 | `app::pane::content::tests::pane_markers_delegates_to_content` | determinística |
| 98 | `app::pane::content::tests::pane_price_lines_delegates_to_content` | determinística |
| 99 | `app::pane::divider::tests::divider_clone` | determinística |
| 100 | `app::pane::divider::tests::divider_creation` | determinística |
| 101 | `app::pane::divider::tests::divider_hit_rect` | determinística |
| 102 | `app::pane::divider::tests::divider_hit_test_inside` | determinística |
| 103 | `app::pane::divider::tests::divider_hit_test_outside` | determinística |
| 104 | `app::pane::divider::tests::divider_rect` | determinística |
| 105 | `app::pane::divider::tests::divider_with_dimensions` | determinística |
| 106 | `app::pane::events::tests::bus_starts_empty` | determinística |
| 107 | `app::pane::events::tests::clone_and_debug` | determinística |
| 108 | `app::pane::events::tests::divider_dragged` | determinística |
| 109 | `app::pane::events::tests::drain_on_empty_bus` | determinística |
| 110 | `app::pane::events::tests::drain_returns_all_and_clears` | determinística |
| 111 | `app::pane::events::tests::pane_added_and_removed` | determinística |
| 112 | `app::pane::events::tests::pane_resized` | determinística |
| 113 | `app::pane::events::tests::push_increments_len` | determinística |
| 114 | `app::pane::tests::add_indicator` | determinística |
| 115 | `app::pane::tests::add_multiple_series` | determinística |
| 116 | `app::pane::tests::add_series` | determinística |
| 117 | `app::pane::tests::ensure_price_scales_populates_empty` | determinística |
| 118 | `app::pane::tests::pane_add_overlay_scale` | determinística |
| 119 | `app::pane::tests::pane_clear_layers` | determinística |
| 120 | `app::pane::tests::pane_creation` | determinística |
| 121 | `app::pane::tests::pane_default_viewport` | determinística |
| 122 | `app::pane::tests::pane_drawings_empty_by_default` | determinística |
| 123 | `app::pane::tests::pane_formatter_accessor` | determinística |
| 124 | `app::pane::tests::pane_has_left_and_right_scales` | determinística |
| 125 | `app::pane::tests::pane_layers_empty_by_default` | determinística |
| 126 | `app::pane::tests::pane_markers_accessor` | determinística |
| 127 | `app::pane::tests::pane_markers_mut_accessor` | determinística |
| 128 | `app::pane::tests::pane_price_lines_accessor` | determinística |
| 129 | `app::pane::tests::pane_price_lines_mut_accessor` | determinística |
| 130 | `app::pane::tests::pane_price_scale_mut` | determinística |
| 131 | `app::pane::tests::pane_primary_scale_is_left` | determinística |
| 132 | `app::pane::tests::pixel_height_calculation` | determinística |
| 133 | `app::pane::tests::pixel_y_offset_first_pane` | determinística |
| 134 | `app::pane::tests::pixel_y_offset_second_pane` | determinística |
| 135 | `app::pane::tests::series_defaults_to_left_scale` | determinística |
| 136 | `app::viewport_bounds::tests::clamp_zoom_clamps_to_max` | determinística |
| 137 | `app::viewport_bounds::tests::clamp_zoom_clamps_to_min` | determinística |
| 138 | `app::viewport_bounds::tests::clamp_zoom_custom_bounds` | determinística |
| 139 | `app::viewport_bounds::tests::clamp_zoom_identity_factor` | determinística |
| 140 | `app::viewport_bounds::tests::clamp_zoom_within_bounds` | determinística |
| 141 | `app::viewport_bounds::tests::default_bounds` | determinística |
| 142 | `app::viewport_interaction::tests::handle_input_sets_crosshair` | determinística |
| 143 | `app::viewport_interaction::tests::handle_input_updates_viewport` | determinística |
| 144 | `app::viewport_interaction::tests::handle_input_zoom_at_cursor` | determinística |
| 145 | `app::viewport_management::tests::apply_pan_backward` | determinística |
| 146 | `app::viewport_management::tests::apply_pan_forward` | determinística |
| 147 | `app::viewport_management::tests::apply_pan_saturates_at_zero` | determinística |
| 148 | `app::viewport_management::tests::apply_zoom_clamps_to_max` | determinística |
| 149 | `app::viewport_management::tests::apply_zoom_clamps_to_min` | determinística |
| 150 | `app::viewport_management::tests::apply_zoom_in` | determinística |
| 151 | `app::viewport_management::tests::apply_zoom_out` | determinística |
| 152 | `app::viewport_management::tests::auto_fit_empty_bars` | determinística |
| 153 | `app::viewport_management::tests::auto_fit_sets_range` | determinística |
| 154 | `app::viewport_management::tests::auto_fit_single_bar` | determinística |
| 155 | `app::viewport_management::tests::create_linear_scale_from_viewport` | determinística |
| 156 | `app::viewport_management::tests::create_time_scale_from_viewport` | determinística |
| 157 | `app::viewport_management::tests::new_has_sensible_defaults` | determinística |
| 158 | `app::viewport_management::tests::scale_roundtrip_after_auto_fit` | determinística |
| 159 | `app::viewport_management::tests::zoom_preserves_center` | determinística |
| 160 | `builder::tests::builder_build_produces_config` | determinística |
| 161 | `builder::tests::builder_chained` | determinística |
| 162 | `builder::tests::builder_dimensions` | determinística |
| 163 | `builder::tests::builder_new` | determinística |
| 164 | `builder::tests::builder_pane` | determinística |
| 165 | `builder::tests::builder_theme` | determinística |
| 166 | `builder::tests::builder_title` | determinística |
| 167 | `builder::tests::config_theme_mut` | determinística |
| 168 | `series::line_break::tests::line_break_block_new` | determinística |
| 169 | `series::line_break::tests::line_break_build_empty` | determinística |
| 170 | `series::line_break::tests::line_break_build_single_bar` | determinística |
| 171 | `series::line_break::tests::line_break_hit_test_empty` | determinística |
| 172 | `series::line_break::tests::line_break_series_default` | determinística |
| 173 | `series::line_break::tests::line_break_series_empty_no_commands` | determinística |
| 174 | `series::line_break::tests::line_break_series_generates_rect_commands` | determinística |
| 175 | `series::line_break::tests::line_break_series_hit_test` | determinística |
| 176 | `series::line_break::tests::line_break_series_new` | determinística |
| 177 | `series::line_break::tests::line_break_series_set_blocks` | determinística |
| 178 | `series::line_break::tests::line_break_series_up_down_colors` | determinística |
| 179 | `series::point_figure::tests::pf_build_from_prices_empty` | determinística |
| 180 | `series::point_figure::tests::pf_build_from_prices_monotonic_rise` | determinística |
| 181 | `series::point_figure::tests::pf_column_fall` | determinística |
| 182 | `series::point_figure::tests::pf_column_rise` | determinística |
| 183 | `series::point_figure::tests::pf_series_bands_set_after_update` | determinística |
| 184 | `series::point_figure::tests::pf_series_default` | determinística |
| 185 | `series::point_figure::tests::pf_series_empty_no_commands` | determinística |
| 186 | `series::point_figure::tests::pf_series_generates_rect_commands` | determinística |
| 187 | `series::point_figure::tests::pf_series_hit_test` | determinística |
| 188 | `series::point_figure::tests::pf_series_hit_test_empty` | determinística |
| 189 | `series::point_figure::tests::pf_series_max_boxes` | determinística |
| 190 | `series::point_figure::tests::pf_series_multiple_columns` | determinística |
| 191 | `series::point_figure::tests::pf_series_new` | determinística |
| 192 | `series::point_figure::tests::pf_series_set_columns` | determinística |
| 193 | `series::range::tests::range_bar_new` | determinística |
| 194 | `series::range::tests::range_build_empty` | determinística |
| 195 | `series::range::tests::range_build_zero_range` | determinística |
| 196 | `series::range::tests::range_hit_test_empty` | determinística |
| 197 | `series::range::tests::range_series_bullish_bearish_colors` | determinística |
| 198 | `series::range::tests::range_series_default` | determinística |
| 199 | `series::range::tests::range_series_empty_no_commands` | determinística |
| 200 | `series::range::tests::range_series_generates_rect_commands` | determinística |
| 201 | `series::range::tests::range_series_hit_test` | determinística |
| 202 | `series::range::tests::range_series_new` | determinística |
| 203 | `series::range::tests::range_series_set_data` | determinística |
| 204 | `series::step_line::tests::step_line_bounds_set_after_update` | determinística |
| 205 | `series::step_line::tests::step_line_clone` | determinística |
| 206 | `series::step_line::tests::step_line_commands_are_lines` | determinística |
| 207 | `series::step_line::tests::step_line_default` | determinística |
| 208 | `series::step_line::tests::step_line_empty_data_no_commands` | determinística |
| 209 | `series::step_line::tests::step_line_generates_correct_command_count` | determinística |
| 210 | `series::step_line::tests::step_line_hit_test_empty` | determinística |
| 211 | `series::step_line::tests::step_line_hit_test_returns_nearest` | determinística |
| 212 | `series::step_line::tests::step_line_new` | determinística |
| 213 | `series::step_line::tests::step_line_set_data` | determinística |
| 214 | `series::step_line::tests::step_line_single_point_no_commands` | determinística |
| 215 | `series::step_line::tests::step_line_two_points_one_horizontal_one_vertical` | determinística |
| 216 | `series::step_line::tests::step_line_value_range` | determinística |
| 217 | `series::step_line::tests::step_point_new` | determinística |
| 218 | `series::volume::tests::volume_bar_new` | determinística |
| 219 | `series::volume::tests::volume_series_bounds_set_after_update` | determinística |
| 220 | `series::volume::tests::volume_series_bullish_and_bearish_colors` | determinística |
| 221 | `series::volume::tests::volume_series_default` | determinística |
| 222 | `series::volume::tests::volume_series_empty_max_volume` | determinística |
| 223 | `series::volume::tests::volume_series_empty_no_commands` | determinística |
| 224 | `series::volume::tests::volume_series_generates_rect_commands` | determinística |
| 225 | `series::volume::tests::volume_series_hit_test` | determinística |
| 226 | `series::volume::tests::volume_series_hit_test_empty` | determinística |
| 227 | `series::volume::tests::volume_series_layer_z_index` | determinística |
| 228 | `series::volume::tests::volume_series_max_volume` | determinística |
| 229 | `series::volume::tests::volume_series_new` | determinística |
| 230 | `series::volume::tests::volume_series_set_data` | determinística |
| 231 | `series::volume::tests::volume_series_zero_volume_no_commands` | determinística |
| 232 | `pixel_perfect_rect_approx` | determinística |
| 233 | `pixel_perfect_snap_approx` | determinística |
| 234 | `snap_generic_f32` | determinística |
| 235 | `snap_generic_f64` | determinística |
| 236 | `auto_scroll_with_pan_controller` | determinística |
| 237 | `crosshair_persists_through_zoom` | determinística |
| 238 | `crosshair_sync_groups` | determinística |
| 239 | `drawing_tool_then_cancel` | determinística |
| 240 | `engine_wheel_zoom_produces_correct_factor` | determinística |
| 241 | `flick_starts_momentum` | determinística |
| 242 | `follow_price_with_zoom` | determinística |
| 243 | `interaction_engine_full_cycle` | determinística |
| 244 | `keyboard_escape_cancel` | determinística |
| 245 | `keyboard_pan_arrows` | determinística |
| 246 | `keyboard_shortcuts_preset_matches_engine` | determinística |
| 247 | `keyboard_zoom_in_out` | determinística |
| 248 | `magnetic_snap_after_zoom` | determinística |
| 249 | `momentum_after_drag` | determinística |
| 250 | `momentum_stops_eventually` | determinística |
| 251 | `pan_then_zoom` | determinística |
| 252 | `pinch_zoom_two_fingers` | determinística |
| 253 | `zoom_mode_switch` | determinística |
| 254 | `zoom_preserves_viewport_size` | determinística |
| 255 | `zoom_then_pan` | determinística |
| 256 | `chart_controller_crosshair_lifecycle` | determinística |
| 257 | `chart_controller_kinetic_decelerates` | determinística |
| 258 | `chart_controller_kinetic_stop_on_zoom` | determinística |
| 259 | `chart_controller_kinetic_update` | determinística |
| 260 | `chart_controller_pan_and_zoom` | determinística |
| 261 | `chart_controller_tick_processes_bars` | determinística |
| 262 | `chart_controller_tick_updates_viewport_on_first_data` | determinística |
| 263 | `divider_drag_resize_integration` | determinística |
| 264 | `full_pipeline_data_to_render` | determinística |
| 265 | `full_pipeline_domain_sets_on_pane` | determinística |
| 266 | `full_pipeline_kinetic_then_render` | determinística |
| 267 | `indicator_renderer_overlay_integration` | determinística |
| 268 | `indicator_renderer_separate_integration` | determinística |
| 269 | `kinetic_scroll_custom_friction` | determinística |
| 270 | `kinetic_scroll_negative_velocity` | determinística |
| 271 | `kinetic_scroll_start_and_update` | determinística |
| 272 | `kinetic_scroll_stop` | determinística |
| 273 | `kinetic_scroll_stops_at_threshold` | determinística |
| 274 | `layout_manager_add_remove_pane` | determinística |
| 275 | `layout_manager_default_pane_structure` | determinística |
| 276 | `layout_manager_divider_drag` | determinística |
| 277 | `layout_manager_min_height_enforced` | determinística |
| 278 | `layout_manager_pane_with_markers_and_lines` | determinística |
| 279 | `layout_manager_sync_time_across_panes` | determinística |
| 280 | `linebreak_block_methods` | determinística |
| 281 | `linebreak_build_from_bars` | determinística |
| 282 | `linebreak_set_blocks_and_update` | determinística |
| 283 | `markers_added_to_series_and_retrieved` | determinística |
| 284 | `markers_builder_chaining` | determinística |
| 285 | `markers_filtered_by_scale` | determinística |
| 286 | `markers_remove_by_id` | determinística |
| 287 | `multi_pane_proportional_heights` | determinística |
| 288 | `multi_pane_vertical_stack_rects` | determinística |
| 289 | `multi_pane_with_gap` | determinística |
| 290 | `overlay_mode_default_overlay_on_pane_0` | determinística |
| 291 | `overlay_mode_separate_pane_integration` | determinística |
| 292 | `pane_add_remove_roundtrip` | determinística |
| 293 | `point_figure_build_from_prices` | determinística |
| 294 | `point_figure_column_types` | determinística |
| 295 | `point_figure_set_columns_and_update` | determinística |
| 296 | `price_formatter_explicit_decimals` | determinística |
| 297 | `price_formatter_format_price` | determinística |
| 298 | `price_formatter_format_short` | determinística |
| 299 | `price_formatter_nan_and_infinity` | determinística |
| 300 | `price_lines_added_and_retrieved` | determinística |
| 301 | `price_lines_builder_chaining` | determinística |
| 302 | `price_lines_filtered_by_scale` | determinística |
| 303 | `price_lines_remove_by_id` | determinística |
| 304 | `price_scale_auto_fit_with_margins_integration` | determinística |
| 305 | `price_scale_locked_no_autofit_integration` | determinística |
| 306 | `range_build_from_bars` | determinística |
| 307 | `range_custom_colors` | determinística |
| 308 | `range_set_data_and_update` | determinística |
| 309 | `seriestype_all_count_matches_impl_count` | determinística |
| 310 | `seriestype_all_display_names_are_unique` | determinística |
| 311 | `stepline_empty_data` | determinística |
| 312 | `stepline_hit_test_bounds` | determinística |
| 313 | `stepline_set_data_and_update` | determinística |
| 314 | `time_scale_scroll_to_end_integration` | determinística |
| 315 | `viewport_time_range_sync_across_panes` | determinística |
| 316 | `volume_max_volume` | determinística |
| 317 | `volume_set_data_and_update` | determinística |
| 318 | `volume_z_index_is_500` | determinística |
| 319 | `rgba_fields_in_0_1` | determinística |
| 320 | `snap_generic_matches_snap` | determinística |
| 321 | `snap_never_negative` | determinística |
| 322 | `snap_size_non_negative` | determinística |
| 323 | `viewport_min_le_max` | determinística |
| 324 | `fc-app/src/builder.rs - builder (line 3)` | determinística |
| 325 | `fc-app/src/theme/mod.rs - theme (line 11)` | determinística |

---

### fc-cache (41 tests)

**Determinísticos:** 41 | **Frágiles:** 0

| # | Test | Clasificación |
|---|------|---------------|
| 326 | `axis::tests::clear_empties_cache` | determinística |
| 327 | `axis::tests::eviction_when_full` | determinística |
| 328 | `axis::tests::get_missing_returns_none` | determinística |
| 329 | `axis::tests::hit_rate_tracks_correctly` | determinística |
| 330 | `axis::tests::insert_and_get` | determinística |
| 331 | `axis::tests::new_cache_is_empty` | determinística |
| 332 | `cache::tests::clear_empties_cache_and_resets_stats` | determinística |
| 333 | `cache::tests::eviction_removes_oldest` | determinística |
| 334 | `cache::tests::get_missing_returns_none` | determinística |
| 335 | `cache::tests::hit_rate_tracks_correctly` | determinística |
| 336 | `cache::tests::insert_and_get` | determinística |
| 337 | `cache::tests::invalidate_removes_entry` | determinística |
| 338 | `cache::tests::invalidate_returns_false_for_missing` | determinística |
| 339 | `cache::tests::invalidate_where_filters_correctly` | determinística |
| 340 | `cache::tests::new_cache_is_empty` | determinística |
| 341 | `geometry::tests::clear_empties_cache_and_resets_stats` | determinística |
| 342 | `geometry::tests::eviction_removes_oldest_when_full` | determinística |
| 343 | `geometry::tests::get_missing_returns_none` | determinística |
| 344 | `geometry::tests::hit_rate_tracks_correctly` | determinística |
| 345 | `geometry::tests::insert_and_get` | determinística |
| 346 | `geometry::tests::invalidate_removes_entry` | determinística |
| 347 | `geometry::tests::invalidate_returns_false_for_missing` | determinística |
| 348 | `geometry::tests::invalidate_series_removes_all_for_series` | determinística |
| 349 | `geometry::tests::new_cache_is_empty` | determinística |
| 350 | `grid::tests::clear_empties_cache` | determinística |
| 351 | `grid::tests::eviction_when_full` | determinística |
| 352 | `grid::tests::get_missing_returns_none` | determinística |
| 353 | `grid::tests::insert_and_get` | determinística |
| 354 | `grid::tests::new_cache_is_empty` | determinística |
| 355 | `indicator::tests::clear_empties_cache` | determinística |
| 356 | `indicator::tests::eviction_when_full` | determinística |
| 357 | `indicator::tests::get_missing_returns_none` | determinística |
| 358 | `indicator::tests::hit_rate_tracks_correctly` | determinística |
| 359 | `indicator::tests::insert_and_get` | determinística |
| 360 | `indicator::tests::new_cache_is_empty` | determinística |
| 361 | `text::tests::clear_empties_cache` | determinística |
| 362 | `text::tests::eviction_when_full` | determinística |
| 363 | `text::tests::get_missing_returns_none` | determinística |
| 364 | `text::tests::hit_rate_tracks_correctly` | determinística |
| 365 | `text::tests::insert_and_get` | determinística |
| 366 | `text::tests::new_cache_is_empty` | determinística |

---

### fc-domain (316 tests)

**Determinísticos:** 315 | **Frágiles:** 1

| # | Test | Clasificación |
|---|------|---------------|
| 367 | `crosshair::magnet_tests::crosshair_update_with_magnet` | determinística |
| 368 | `crosshair::magnet_tests::find_nearest_bar_after_last` | determinística |
| 369 | `crosshair::magnet_tests::find_nearest_bar_before_first` | determinística |
| 370 | `crosshair::magnet_tests::find_nearest_bar_between` | determinística |
| 371 | `crosshair::magnet_tests::find_nearest_bar_empty` | determinística |
| 372 | `crosshair::magnet_tests::find_nearest_bar_exact` | determinística |
| 373 | `crosshair::magnet_tests::snap_to_extreme_mode` | determinística |
| 374 | `crosshair::magnet_tests::snap_to_ohlc_mode` | determinística |
| 375 | `crosshair::magnet_tests::snap_to_ohlc_off` | determinística |
| 376 | `crosshair::tests::deactivate` | determinística |
| 377 | `crosshair::tests::default_is_inactive` | determinística |
| 378 | `crosshair::tests::update_preserves_last_valid_state` | determinística |
| 379 | `crosshair::tests::update_sets_position_and_active` | determinística |
| 380 | `drawing::tests::arrow_builder_methods` | determinística |
| 381 | `drawing::tests::chart_point_new` | determinística |
| 382 | `drawing::tests::drawing_set_add_arrow` | determinística |
| 383 | `drawing::tests::drawing_set_add_ellipse` | determinística |
| 384 | `drawing::tests::drawing_set_add_fibonacci` | determinística |
| 385 | `drawing::tests::drawing_set_add_fibonacci_extension` | determinística |
| 386 | `drawing::tests::drawing_set_add_horizontal_line` | determinística |
| 387 | `drawing::tests::drawing_set_add_path` | determinística |
| 388 | `drawing::tests::drawing_set_add_pitchfork` | determinística |
| 389 | `drawing::tests::drawing_set_add_ray` | determinística |
| 390 | `drawing::tests::drawing_set_add_rectangle` | determinística |
| 391 | `drawing::tests::drawing_set_add_segment` | determinística |
| 392 | `drawing::tests::drawing_set_add_trend_line` | determinística |
| 393 | `drawing::tests::drawing_set_add_vertical_line` | determinística |
| 394 | `drawing::tests::drawing_set_all_arrows` | determinística |
| 395 | `drawing::tests::drawing_set_all_ellipses` | determinística |
| 396 | `drawing::tests::drawing_set_all_fibonacci_extensions` | determinística |
| 397 | `drawing::tests::drawing_set_all_paths` | determinística |
| 398 | `drawing::tests::drawing_set_all_pitchforks` | determinística |
| 399 | `drawing::tests::drawing_set_all_rays` | determinística |
| 400 | `drawing::tests::drawing_set_all_rectangles` | determinística |
| 401 | `drawing::tests::drawing_set_all_segments` | determinística |
| 402 | `drawing::tests::drawing_set_is_empty_after_removing_last` | determinística |
| 403 | `drawing::tests::drawing_set_len_counts_all_types` | determinística |
| 404 | `drawing::tests::drawing_set_mixed_all_types` | determinística |
| 405 | `drawing::tests::drawing_set_mixed_with_ellipse_and_path` | determinística |
| 406 | `drawing::tests::drawing_set_mixed_with_new_types` | determinística |
| 407 | `drawing::tests::drawing_set_remove_arrow` | determinística |
| 408 | `drawing::tests::drawing_set_remove_ellipse` | determinística |
| 409 | `drawing::tests::drawing_set_remove_fibonacci` | determinística |
| 410 | `drawing::tests::drawing_set_remove_fibonacci_extension` | determinística |
| 411 | `drawing::tests::drawing_set_remove_from_mixed` | determinística |
| 412 | `drawing::tests::drawing_set_remove_horizontal_line` | determinística |
| 413 | `drawing::tests::drawing_set_remove_nonexistent` | determinística |
| 414 | `drawing::tests::drawing_set_remove_path` | determinística |
| 415 | `drawing::tests::drawing_set_remove_pitchfork` | determinística |
| 416 | `drawing::tests::drawing_set_remove_ray` | determinística |
| 417 | `drawing::tests::drawing_set_remove_rectangle` | determinística |
| 418 | `drawing::tests::drawing_set_remove_segment` | determinística |
| 419 | `drawing::tests::drawing_set_remove_trend_line` | determinística |
| 420 | `drawing::tests::drawing_set_remove_vertical_line` | determinística |
| 421 | `drawing::tests::drawing_set_starts_empty` | determinística |
| 422 | `drawing::tests::ellipse_bounding_box` | determinística |
| 423 | `drawing::tests::ellipse_bounding_box_center_at_zero` | determinística |
| 424 | `drawing::tests::ellipse_builder` | determinística |
| 425 | `drawing::tests::ellipse_clone` | determinística |
| 426 | `drawing::tests::ellipse_contains_beyond_boundary` | determinística |
| 427 | `drawing::tests::ellipse_contains_center` | determinística |
| 428 | `drawing::tests::ellipse_contains_inside` | determinística |
| 429 | `drawing::tests::ellipse_contains_on_boundary` | determinística |
| 430 | `drawing::tests::ellipse_contains_outside` | determinística |
| 431 | `drawing::tests::ellipse_new_defaults` | determinística |
| 432 | `drawing::tests::ellipse_zero_radii` | determinística |
| 433 | `drawing::tests::fibonacci_builder` | determinística |
| 434 | `drawing::tests::fibonacci_clone` | determinística |
| 435 | `drawing::tests::fibonacci_extension_builder` | determinística |
| 436 | `drawing::tests::fibonacci_extension_clone` | determinística |
| 437 | `drawing::tests::fibonacci_extension_level_prices_count` | determinística |
| 438 | `drawing::tests::fibonacci_extension_level_prices_custom` | determinística |
| 439 | `drawing::tests::fibonacci_extension_new_defaults` | determinística |
| 440 | `drawing::tests::fibonacci_extension_price_at_level` | determinística |
| 441 | `drawing::tests::fibonacci_extension_price_at_level_downtrend` | determinística |
| 442 | `drawing::tests::fibonacci_extension_zero_range` | determinística |
| 443 | `drawing::tests::fibonacci_level_prices_count` | determinística |
| 444 | `drawing::tests::fibonacci_level_prices_custom` | determinística |
| 445 | `drawing::tests::fibonacci_new_defaults` | determinística |
| 446 | `drawing::tests::fibonacci_price_at_level` | determinística |
| 447 | `drawing::tests::fibonacci_price_at_level_downtrend` | determinística |
| 448 | `drawing::tests::fibonacci_zero_range` | determinística |
| 449 | `drawing::tests::horizontal_line_builder` | determinística |
| 450 | `drawing::tests::horizontal_line_new_defaults` | determinística |
| 451 | `drawing::tests::move_ellipse` | determinística |
| 452 | `drawing::tests::move_fibonacci_extension` | determinística |
| 453 | `drawing::tests::move_fibonacci_retracement` | frágil |
| 454 | `drawing::tests::move_nonexistent_returns_false` | determinística |
| 455 | `drawing::tests::move_pitchfork` | determinística |
| 456 | `drawing::tests::path_builder` | determinística |
| 457 | `drawing::tests::path_clone` | determinística |
| 458 | `drawing::tests::path_new_defaults` | determinística |
| 459 | `drawing::tests::path_point_access` | determinística |
| 460 | `drawing::tests::path_push` | determinística |
| 461 | `drawing::tests::path_segment_count_closed` | determinística |
| 462 | `drawing::tests::path_segment_count_empty` | determinística |
| 463 | `drawing::tests::path_segment_count_open` | determinística |
| 464 | `drawing::tests::path_segment_count_single` | determinística |
| 465 | `drawing::tests::path_total_length_closed` | determinística |
| 466 | `drawing::tests::path_total_length_empty` | determinística |
| 467 | `drawing::tests::path_total_length_open` | determinística |
| 468 | `drawing::tests::path_total_length_single_point` | determinística |
| 469 | `drawing::tests::pitchfork_asymmetric_b_c` | determinística |
| 470 | `drawing::tests::pitchfork_builder` | determinística |
| 471 | `drawing::tests::pitchfork_clone` | determinística |
| 472 | `drawing::tests::pitchfork_lower_at_a` | determinística |
| 473 | `drawing::tests::pitchfork_lower_at_midpoint` | determinística |
| 474 | `drawing::tests::pitchfork_median_at_a` | determinística |
| 475 | `drawing::tests::pitchfork_median_at_midpoint` | determinística |
| 476 | `drawing::tests::pitchfork_median_interpolation` | determinística |
| 477 | `drawing::tests::pitchfork_new_defaults` | determinística |
| 478 | `drawing::tests::pitchfork_past_a` | determinística |
| 479 | `drawing::tests::pitchfork_upper_at_a` | determinística |
| 480 | `drawing::tests::pitchfork_upper_at_midpoint` | determinística |
| 481 | `drawing::tests::pitchfork_zero_span` | determinística |
| 482 | `drawing::tests::ray_builder_methods` | determinística |
| 483 | `drawing::tests::rectangle_builder` | determinística |
| 484 | `drawing::tests::rectangle_clone` | determinística |
| 485 | `drawing::tests::rectangle_height_price` | determinística |
| 486 | `drawing::tests::rectangle_height_price_reversed_corners` | determinística |
| 487 | `drawing::tests::rectangle_new_defaults` | determinística |
| 488 | `drawing::tests::rectangle_width_ts` | determinística |
| 489 | `drawing::tests::rectangle_width_ts_reversed_corners` | determinística |
| 490 | `drawing::tests::rectangle_zero_size` | determinística |
| 491 | `drawing::tests::segment_builder_methods` | determinística |
| 492 | `drawing::tests::trend_line_builder` | determinística |
| 493 | `drawing::tests::trend_line_clone` | determinística |
| 494 | `drawing::tests::trend_line_new_defaults` | determinística |
| 495 | `drawing::tests::vertical_line_builder` | determinística |
| 496 | `drawing::tests::vertical_line_new_defaults` | determinística |
| 497 | `indicator::tests::default_overlay_mode_is_overlay_on_pane_0` | determinística |
| 498 | `indicator::tests::default_preferred_scale_is_normal` | determinística |
| 499 | `indicator::tests::indicator_calculate_returns_empty` | determinística |
| 500 | `indicator::tests::indicator_name` | determinística |
| 501 | `indicator::tests::log_indicator_preferred_scale` | determinística |
| 502 | `indicator::tests::overlay_mode_clone` | determinística |
| 503 | `indicator::tests::overlay_mode_debug` | determinística |
| 504 | `indicator::tests::separate_pane_indicator` | determinística |
| 505 | `indicator::tests::trait_is_send_sync` | determinística |
| 506 | `indicators::adx::tests::adx_basic` | determinística |
| 507 | `indicators::adx::tests::adx_bounds` | determinística |
| 508 | `indicators::adx::tests::adx_exact_period` | determinística |
| 509 | `indicators::adx::tests::adx_insufficient_data` | determinística |
| 510 | `indicators::adx::tests::adx_name` | determinística |
| 511 | `indicators::adx::tests::adx_no_trend` | determinística |
| 512 | `indicators::adx::tests::adx_strong_trend` | determinística |
| 513 | `indicators::atr::tests::atr_always_non_negative` | determinística |
| 514 | `indicators::atr::tests::atr_basic` | determinística |
| 515 | `indicators::atr::tests::atr_constant_range` | determinística |
| 516 | `indicators::atr::tests::atr_exact_period` | determinística |
| 517 | `indicators::atr::tests::atr_insufficient_data` | determinística |
| 518 | `indicators::atr::tests::atr_name` | determinística |
| 519 | `indicators::bollinger::tests::bollinger_band_ordering` | determinística |
| 520 | `indicators::bollinger::tests::bollinger_default_params` | determinística |
| 521 | `indicators::bollinger::tests::bollinger_insufficient_data` | determinística |
| 522 | `indicators::bollinger::tests::bollinger_middle_matches_sma` | determinística |
| 523 | `indicators::bollinger::tests::bollinger_name` | determinística |
| 524 | `indicators::cci::tests::cci_basic` | determinística |
| 525 | `indicators::cci::tests::cci_constant_prices` | determinística |
| 526 | `indicators::cci::tests::cci_default_period` | determinística |
| 527 | `indicators::cci::tests::cci_exact_period` | determinística |
| 528 | `indicators::cci::tests::cci_insufficient_data` | determinística |
| 529 | `indicators::cci::tests::cci_name` | determinística |
| 530 | `indicators::cci::tests::cci_single_bar` | determinística |
| 531 | `indicators::ema::tests::ema_basic` | determinística |
| 532 | `indicators::ema::tests::ema_converges_to_constant_input` | determinística |
| 533 | `indicators::ema::tests::ema_first_value_matches_sma` | determinística |
| 534 | `indicators::ema::tests::ema_insufficient_data` | determinística |
| 535 | `indicators::ema::tests::ema_name` | determinística |
| 536 | `indicators::heikin_ashi::tests::empty_series` | determinística |
| 537 | `indicators::heikin_ashi::tests::first_ha_close_formula` | determinística |
| 538 | `indicators::heikin_ashi::tests::flat_prices` | determinística |
| 539 | `indicators::heikin_ashi::tests::name_returns_correct_string` | determinística |
| 540 | `indicators::heikin_ashi::tests::output_count_matches_input` | determinística |
| 541 | `indicators::heikin_ashi::tests::second_bar_uses_ha_open` | determinística |
| 542 | `indicators::heikin_ashi::tests::single_bar` | determinística |
| 543 | `indicators::heikin_ashi::tests::trend_smoothing` | determinística |
| 544 | `indicators::ichimoku::tests::ichimoku_default_params` | determinística |
| 545 | `indicators::ichimoku::tests::ichimoku_insufficient_data` | determinística |
| 546 | `indicators::ichimoku::tests::ichimoku_kijun_length` | determinística |
| 547 | `indicators::ichimoku::tests::ichimoku_name` | determinística |
| 548 | `indicators::ichimoku::tests::ichimoku_tenkan_length` | determinística |
| 549 | `indicators::ichimoku::tests::ichimoku_tenkan_midpoint` | determinística |
| 550 | `indicators::kagi::tests::kagi_atr_mode` | determinística |
| 551 | `indicators::kagi::tests::kagi_atr_mode_insufficient_bars` | determinística |
| 552 | `indicators::kagi::tests::kagi_basic` | determinística |
| 553 | `indicators::kagi::tests::kagi_downward_reversal` | determinística |
| 554 | `indicators::kagi::tests::kagi_empty_series` | determinística |
| 555 | `indicators::kagi::tests::kagi_exact_period` | determinística |
| 556 | `indicators::kagi::tests::kagi_flat_prices_no_reversal` | determinística |
| 557 | `indicators::kagi::tests::kagi_insufficient_data_for_atr` | determinística |
| 558 | `indicators::kagi::tests::kagi_large_reversal` | determinística |
| 559 | `indicators::kagi::tests::kagi_name` | determinística |
| 560 | `indicators::kagi::tests::kagi_single_bar` | determinística |
| 561 | `indicators::kagi::tests::kagi_trend_continuation` | determinística |
| 562 | `indicators::kagi::tests::kagi_upward_reversal` | determinística |
| 563 | `indicators::macd::tests::macd_default_params` | determinística |
| 564 | `indicators::macd::tests::macd_full_result_consistency` | determinística |
| 565 | `indicators::macd::tests::macd_histogram` | determinística |
| 566 | `indicators::macd::tests::macd_insufficient_data` | determinística |
| 567 | `indicators::macd::tests::macd_name` | determinística |
| 568 | `indicators::parabolic_sar::tests::parabolic_sar_basic` | determinística |
| 569 | `indicators::parabolic_sar::tests::parabolic_sar_custom_params` | determinística |
| 570 | `indicators::parabolic_sar::tests::parabolic_sar_default_params` | determinística |
| 571 | `indicators::parabolic_sar::tests::parabolic_sar_downtrend` | determinística |
| 572 | `indicators::parabolic_sar::tests::parabolic_sar_insufficient_data` | determinística |
| 573 | `indicators::parabolic_sar::tests::parabolic_sar_name` | determinística |
| 574 | `indicators::parabolic_sar::tests::parabolic_sar_reversal` | determinística |
| 575 | `indicators::parabolic_sar::tests::parabolic_sar_two_bars` | determinística |
| 576 | `indicators::parabolic_sar::tests::parabolic_sar_uptrend` | determinística |
| 577 | `indicators::renko::tests::renko_atr_mode` | determinística |
| 578 | `indicators::renko::tests::renko_atr_mode_insufficient_bars` | determinística |
| 579 | `indicators::renko::tests::renko_basic` | determinística |
| 580 | `indicators::renko::tests::renko_bearish_bricks` | determinística |
| 581 | `indicators::renko::tests::renko_direction_change` | determinística |
| 582 | `indicators::renko::tests::renko_empty_series` | determinística |
| 583 | `indicators::renko::tests::renko_exact_period` | determinística |
| 584 | `indicators::renko::tests::renko_fixed_brick_size` | determinística |
| 585 | `indicators::renko::tests::renko_flat_prices_no_new_bricks` | determinística |
| 586 | `indicators::renko::tests::renko_insufficient_data_for_atr` | determinística |
| 587 | `indicators::renko::tests::renko_large_move` | determinística |
| 588 | `indicators::renko::tests::renko_name` | determinística |
| 589 | `indicators::renko::tests::renko_single_bar` | determinística |
| 590 | `indicators::rsi::tests::rsi_all_gains` | determinística |
| 591 | `indicators::rsi::tests::rsi_all_losses` | determinística |
| 592 | `indicators::rsi::tests::rsi_bounds` | determinística |
| 593 | `indicators::rsi::tests::rsi_insufficient_data` | determinística |
| 594 | `indicators::rsi::tests::rsi_name` | determinística |
| 595 | `indicators::sma::tests::sma_basic` | determinística |
| 596 | `indicators::sma::tests::sma_exact_period` | determinística |
| 597 | `indicators::sma::tests::sma_insufficient_data` | determinística |
| 598 | `indicators::sma::tests::sma_name` | determinística |
| 599 | `indicators::sma::tests::sma_single_period_rolling` | determinística |
| 600 | `indicators::stochastic::tests::stochastic_d_bounds` | determinística |
| 601 | `indicators::stochastic::tests::stochastic_d_length` | determinística |
| 602 | `indicators::stochastic::tests::stochastic_flat_market` | determinística |
| 603 | `indicators::stochastic::tests::stochastic_insufficient_data` | determinística |
| 604 | `indicators::stochastic::tests::stochastic_k_bounds` | determinística |
| 605 | `indicators::stochastic::tests::stochastic_name` | determinística |
| 606 | `indicators::supertrend::tests::basic_output_count` | determinística |
| 607 | `indicators::supertrend::tests::default_values` | determinística |
| 608 | `indicators::supertrend::tests::exact_period_plus_one` | determinística |
| 609 | `indicators::supertrend::tests::flat_prices` | determinística |
| 610 | `indicators::supertrend::tests::insufficient_data` | determinística |
| 611 | `indicators::supertrend::tests::name_returns_correct_string` | determinística |
| 612 | `indicators::supertrend::tests::output_values_are_positive` | determinística |
| 613 | `indicators::supertrend::tests::single_bar` | determinística |
| 614 | `indicators::supertrend::tests::trend_reversal` | determinística |
| 615 | `indicators::vwap::tests::vwap_basic` | determinística |
| 616 | `indicators::vwap::tests::vwap_cumulative_weighted` | determinística |
| 617 | `indicators::vwap::tests::vwap_empty_series` | determinística |
| 618 | `indicators::vwap::tests::vwap_exact_period` | determinística |
| 619 | `indicators::vwap::tests::vwap_name` | determinística |
| 620 | `indicators::vwap::tests::vwap_session_reset` | determinística |
| 621 | `indicators::vwap::tests::vwap_single_bar` | determinística |
| 622 | `indicators::vwap::tests::vwap_zero_volume` | determinística |
| 623 | `indicators::williams_r::tests::williams_r_basic` | determinística |
| 624 | `indicators::williams_r::tests::williams_r_bounds` | determinística |
| 625 | `indicators::williams_r::tests::williams_r_constant_prices` | determinística |
| 626 | `indicators::williams_r::tests::williams_r_default_period` | determinística |
| 627 | `indicators::williams_r::tests::williams_r_exact_period` | determinística |
| 628 | `indicators::williams_r::tests::williams_r_insufficient_data` | determinística |
| 629 | `indicators::williams_r::tests::williams_r_name` | determinística |
| 630 | `indicators::williams_r::tests::williams_r_overbought` | determinística |
| 631 | `indicators::williams_r::tests::williams_r_oversold` | determinística |
| 632 | `indicators::williams_r::tests::williams_r_single_bar` | determinística |
| 633 | `marker::tests::marker_builder` | determinística |
| 634 | `marker::tests::marker_new` | determinística |
| 635 | `marker::tests::marker_set_add_remove` | determinística |
| 636 | `marker::tests::marker_set_in_range` | determinística |
| 637 | `price_line::tests::price_line_builder` | determinística |
| 638 | `price_line::tests::price_line_new` | determinística |
| 639 | `price_line::tests::price_line_set_add_remove` | determinística |
| 640 | `price_line::tests::price_line_set_for_scale` | determinística |
| 641 | `price_scale::tests::all_modes_exist` | determinística |
| 642 | `price_scale::tests::auto_fit_disabled_noop` | determinística |
| 643 | `price_scale::tests::auto_fit_locked_noop` | determinística |
| 644 | `price_scale::tests::auto_fit_padds_range` | determinística |
| 645 | `price_scale::tests::auto_fit_with_margins` | determinística |
| 646 | `price_scale::tests::auto_fit_zero_range` | determinística |
| 647 | `price_scale::tests::contains_inside_range` | determinística |
| 648 | `price_scale::tests::default_format_auto_detects_precision` | determinística |
| 649 | `price_scale::tests::default_mode_is_normal` | determinística |
| 650 | `price_scale::tests::explicit_format` | determinística |
| 651 | `price_scale::tests::format_infinity` | determinística |
| 652 | `price_scale::tests::format_nan` | determinística |
| 653 | `price_scale::tests::format_short_small_prices` | determinística |
| 654 | `price_scale::tests::format_short_uses_k_suffix` | determinística |
| 655 | `price_scale::tests::is_editable_locked` | determinística |
| 656 | `price_scale::tests::is_editable_normal` | determinística |
| 657 | `price_scale::tests::left_and_right_are_distinct` | determinística |
| 658 | `price_scale::tests::overlay_equality_by_name` | determinística |
| 659 | `price_scale::tests::overlay_inequality_by_name` | determinística |
| 660 | `price_scale::tests::set_mode_changes_mode` | determinística |
| 661 | `viewport::tests::contains_time_at_boundary` | determinística |
| 662 | `viewport::tests::contains_time_inside` | determinística |
| 663 | `viewport::tests::contains_time_outside` | determinística |
| 664 | `viewport::tests::default_viewport` | determinística |
| 665 | `viewport::tests::pan_backward` | determinística |
| 666 | `viewport::tests::pan_forward` | determinística |
| 667 | `viewport::tests::pan_saturate_at_zero` | determinística |
| 668 | `viewport::tests::price_to_y_bottom` | determinística |
| 669 | `viewport::tests::price_to_y_clamps_above` | determinística |
| 670 | `viewport::tests::price_to_y_clamps_below` | determinística |
| 671 | `viewport::tests::price_to_y_midpoint` | determinística |
| 672 | `viewport::tests::price_to_y_top` | determinística |
| 673 | `viewport::tests::price_to_y_zero_range` | determinística |
| 674 | `viewport::tests::y_to_price_roundtrip` | determinística |
| 675 | `viewport::tests::y_to_price_zero_height` | determinística |
| 676 | `viewport::tests::zoom_in` | determinística |
| 677 | `viewport::tests::zoom_out` | determinística |
| 678 | `fc-domain/src/crosshair.rs - crosshair::Crosshair (line 83)` | determinística |
| 679 | `fc-domain/src/indicator.rs - indicator::Indicator (line 19)` | determinística |
| 680 | `fc-domain/src/marker.rs - marker::Marker (line 34)` | determinística |
| 681 | `fc-domain/src/price_line.rs - price_line::PriceLine (line 10)` | determinística |
| 682 | `fc-domain/src/viewport.rs - viewport::Viewport (line 11)` | determinística |

---

### fc-drawing (99 tests)

**Determinísticos:** 99 | **Frágiles:** 0

| # | Test | Clasificación |
|---|------|---------------|
| 683 | `aabb_hit_beyond_tolerance` | determinística |
| 684 | `aabb_hit_inside` | determinística |
| 685 | `aabb_hit_outside` | determinística |
| 686 | `aabb_hit_within_tolerance` | determinística |
| 687 | `all_types_implement_drawing_trait` | determinística |
| 688 | `arrow_bounds` | determinística |
| 689 | `arrow_builder` | determinística |
| 690 | `arrow_construction` | determinística |
| 691 | `arrow_move_by` | determinística |
| 692 | `drawing_bounds_combine` | determinística |
| 693 | `drawing_bounds_combine_absorbs` | determinística |
| 694 | `drawing_bounds_contains` | determinística |
| 695 | `drawing_bounds_contains_on_boundary` | determinística |
| 696 | `drawing_bounds_from_point` | determinística |
| 697 | `drawing_bounds_from_points` | determinística |
| 698 | `drawing_bounds_new` | determinística |
| 699 | `drawing_bounds_price_height` | determinística |
| 700 | `drawing_bounds_time_width` | determinística |
| 701 | `drawing_bounds_time_width_saturating` | determinística |
| 702 | `ellipse_bounds` | determinística |
| 703 | `ellipse_builder` | determinística |
| 704 | `ellipse_construction` | determinística |
| 705 | `ellipse_hit_test_center` | determinística |
| 706 | `ellipse_hit_test_outside` | determinística |
| 707 | `ellipse_move_by` | determinística |
| 708 | `fib_extension_construction` | determinística |
| 709 | `fib_extension_move_by` | determinística |
| 710 | `fib_extension_price_at_level` | determinística |
| 711 | `fib_retracement_builder` | determinística |
| 712 | `fib_retracement_construction` | determinística |
| 713 | `fib_retracement_level_prices` | determinística |
| 714 | `fib_retracement_move_by` | determinística |
| 715 | `fib_retracement_price_at_level` | determinística |
| 716 | `hit_result_variants` | determinística |
| 717 | `horizontal_line_bounds` | determinística |
| 718 | `horizontal_line_builder` | determinística |
| 719 | `horizontal_line_construction` | determinística |
| 720 | `horizontal_line_move_by` | determinística |
| 721 | `image_drawing_builder` | determinística |
| 722 | `image_drawing_construction` | determinística |
| 723 | `image_drawing_move_by` | determinística |
| 724 | `label_drawing_builder` | determinística |
| 725 | `label_drawing_construction` | determinística |
| 726 | `label_drawing_move_by` | determinística |
| 727 | `path_bounds` | determinística |
| 728 | `path_bounds_empty` | determinística |
| 729 | `path_builder` | determinística |
| 730 | `path_construction` | determinística |
| 731 | `path_hit_test_empty` | determinística |
| 732 | `path_hit_test_miss` | determinística |
| 733 | `path_hit_test_multi_segment` | determinística |
| 734 | `path_hit_test_on_segment` | determinística |
| 735 | `path_hit_test_single_point` | determinística |
| 736 | `path_move_by` | determinística |
| 737 | `path_point` | determinística |
| 738 | `path_push` | determinística |
| 739 | `path_segment_count_closed` | determinística |
| 740 | `path_segment_count_empty_or_single` | determinística |
| 741 | `path_segment_count_open` | determinística |
| 742 | `path_total_length` | determinística |
| 743 | `path_total_length_closed` | determinística |
| 744 | `path_total_length_trivial` | determinística |
| 745 | `pitchfork_bounds` | determinística |
| 746 | `pitchfork_construction` | determinística |
| 747 | `pitchfork_hit_test_miss` | determinística |
| 748 | `pitchfork_hit_test_on_median` | determinística |
| 749 | `pitchfork_move_by` | determinística |
| 750 | `ray_bounds` | determinística |
| 751 | `ray_construction` | determinística |
| 752 | `ray_hit_test_degenerate` | determinística |
| 753 | `ray_hit_test_miss` | determinística |
| 754 | `ray_hit_test_on_ray` | determinística |
| 755 | `ray_move_by` | determinística |
| 756 | `rectangle_bounds` | determinística |
| 757 | `rectangle_builder` | determinística |
| 758 | `rectangle_construction` | determinística |
| 759 | `rectangle_dimensions` | determinística |
| 760 | `rectangle_dimensions_inverted` | determinística |
| 761 | `rectangle_move_by` | determinística |
| 762 | `segment_bounds` | determinística |
| 763 | `segment_construction` | determinística |
| 764 | `segment_move_by` | determinística |
| 765 | `text_drawing_bounds_is_point` | determinística |
| 766 | `text_drawing_builder` | determinística |
| 767 | `text_drawing_construction` | determinística |
| 768 | `text_drawing_move_by` | determinística |
| 769 | `trendline_as_any_downcast` | determinística |
| 770 | `trendline_bounds` | determinística |
| 771 | `trendline_builder` | determinística |
| 772 | `trendline_construction` | determinística |
| 773 | `trendline_hit_test_miss` | determinística |
| 774 | `trendline_hit_test_on_line` | determinística |
| 775 | `trendline_move_by` | determinística |
| 776 | `trendline_move_by_saturating_overflow` | determinística |
| 777 | `trendline_selection` | determinística |
| 778 | `type_name_returns_correct_type` | determinística |
| 779 | `vertical_line_bounds` | determinística |
| 780 | `vertical_line_construction` | determinística |
| 781 | `vertical_line_move_by` | determinística |

---

### fc-examples (71 tests)

**Determinísticos:** 71 | **Frágiles:** 0

| # | Test | Clasificación |
|---|------|---------------|
| 782 | `adapters::data::simulated::tests::f64_in_range` | determinística |
| 783 | `adapters::data::simulated::tests::generate_bar_deterministic_with_seed` | determinística |
| 784 | `adapters::data::simulated::tests::generate_bar_links_consecutive_closes` | determinística |
| 785 | `adapters::data::simulated::tests::generate_bar_ohlcv_invariants` | determinística |
| 786 | `adapters::data::simulated::tests::lcg_deterministic` | determinística |
| 787 | `adapters::data::simulated::tests::lcg_different_seeds_differ` | determinística |
| 788 | `adapters::data::simulated::tests::normal_distribution_mean_near_zero` | determinística |
| 789 | `adapters::data::simulated::tests::provider_implements_data_provider_trait` | determinística |
| 790 | `adapters::data::simulated::tests::provider_name` | determinística |
| 791 | `adapters::data::simulated::tests::provider_receiver_available_after_construction` | determinística |
| 792 | `adapters::data::simulated::tests::provider_start_stop` | determinística |
| 793 | `adapters::data::simulated::tests::provider_streams_bars` | determinística |
| 794 | `adapters::data::simulated::tests::simulate_high_vol_wider_range` | determinística |
| 795 | `...sts::area_update_from_state_tests::area_baseline_at_value_min_fills_to_bottom` | determinística |
| 796 | `...te_from_state_tests::area_close_vertex_is_above_baseline_when_price_above_min` | determinística |
| 797 | `...derer_tests::area_update_from_state_tests::area_empty_series_produces_nothing` | determinística |
| 798 | `...ea_update_from_state_tests::area_initialized_with_correct_vertex_index_counts` | determinística |
| 799 | `...erer_tests::area_update_from_state_tests::area_single_bar_produces_no_indices` | determinística |
| 800 | `...update_from_state_tests::area_update_produces_correct_vertex_and_index_counts` | determinística |
| 801 | `...area_renderer_tests::area_update_from_state_tests::area_visible_bar_filtering` | determinística |
| 802 | `...er_tests::area_update_from_state_tests::area_zero_time_range_produces_nothing` | determinística |
| 803 | `...r_tests::area_update_from_state_tests::area_zero_value_range_produces_nothing` | determinística |
| 804 | `...ering::area_renderer_tests::baseline_renderer_tests::baseline_color_selection` | determinística |
| 805 | `...:rendering::area_renderer_tests::baseline_renderer_tests::baseline_empty_bars` | determinística |
| 806 | `...area_renderer_tests::baseline_renderer_tests::baseline_index_count_for_n_bars` | determinística |
| 807 | `...rea_renderer_tests::baseline_renderer_tests::baseline_vertex_count_for_n_bars` | determinística |
| 808 | `...derer_tests::baseline_update_from_state_tests::baseline_color_selection_above` | determinística |
| 809 | `...derer_tests::baseline_update_from_state_tests::baseline_color_selection_below` | determinística |
| 810 | `...sts::baseline_update_from_state_tests::baseline_empty_series_produces_nothing` | determinística |
| 811 | `...:baseline_update_from_state_tests::baseline_initialized_with_correct_capacity` | determinística |
| 812 | `...tests::baseline_update_from_state_tests::baseline_midpoint_at_viewport_center` | determinística |
| 813 | `...tests::baseline_update_from_state_tests::baseline_single_bar_produces_nothing` | determinística |
| 814 | `...te_from_state_tests::baseline_update_produces_correct_vertex_and_index_counts` | determinística |
| 815 | `...derer_tests::baseline_update_from_state_tests::baseline_visible_bar_filtering` | determinística |
| 816 | `...rer_tests::baseline_update_from_state_tests::baseline_y_screen_coords_flipped` | determinística |
| 817 | `...::baseline_update_from_state_tests::baseline_zero_time_range_produces_nothing` | determinística |
| 818 | `...:baseline_update_from_state_tests::baseline_zero_value_range_produces_nothing` | determinística |
| 819 | `...rea_renderer_tests::histogram_renderer_tests::histogram_bar_width_calculation` | determinística |
| 820 | `...ring::area_renderer_tests::histogram_renderer_tests::histogram_baseline_color` | determinística |
| 821 | `...endering::area_renderer_tests::histogram_renderer_tests::histogram_empty_bars` | determinística |
| 822 | `...ing::area_renderer_tests::histogram_renderer_tests::histogram_quad_generation` | determinística |
| 823 | `...endering::area_renderer_tests::histogram_renderer_tests::histogram_single_bar` | determinística |
| 824 | `...ea_renderer_tests::histogram_renderer_tests::histogram_y_baseline_above_value` | determinística |
| 825 | `...ea_renderer_tests::histogram_renderer_tests::histogram_y_baseline_below_value` | determinística |
| 826 | `...renderer_tests::histogram_update_from_state_tests::histogram_baseline_at_zero` | determinística |
| 827 | `...erer_tests::histogram_update_from_state_tests::histogram_baseline_color_above` | determinística |
| 828 | `...erer_tests::histogram_update_from_state_tests::histogram_baseline_color_below` | determinística |
| 829 | `...s::histogram_update_from_state_tests::histogram_empty_series_produces_nothing` | determinística |
| 830 | `...istogram_update_from_state_tests::histogram_initialized_with_correct_capacity` | determinística |
| 831 | `..._tests::histogram_update_from_state_tests::histogram_single_bar_produces_quad` | determinística |
| 832 | `...e_from_state_tests::histogram_update_produces_correct_vertex_and_index_counts` | determinística |
| 833 | `...rer_tests::histogram_update_from_state_tests::histogram_visible_bar_filtering` | determinística |
| 834 | `...ogram_update_from_state_tests::histogram_y_baseline_above_value_when_positive` | determinística |
| 835 | `...ogram_update_from_state_tests::histogram_y_baseline_below_value_when_negative` | determinística |
| 836 | `...histogram_update_from_state_tests::histogram_zero_time_range_produces_nothing` | determinística |
| 837 | `...istogram_update_from_state_tests::histogram_zero_value_range_produces_nothing` | determinística |
| 838 | `adapters::rendering::area_renderer_tests::tests::area_empty_bars` | determinística |
| 839 | `adapters::rendering::area_renderer_tests::tests::area_index_count_for_n_bars` | determinística |
| 840 | `adapters::rendering::area_renderer_tests::tests::area_single_bar_no_indices` | determinística |
| 841 | `adapters::rendering::area_renderer_tests::tests::area_vertex_count_for_n_bars` | determinística |
| 842 | `adapters::rendering::area_renderer_tests::tests::area_x_coordinate_within_canvas` | determinística |
| 843 | `adapters::rendering::area_renderer_tests::tests::area_y_close_within_canvas` | determinística |
| 844 | `...ters::rendering::area_renderer_tests::tests::area_zero_range_produces_nothing` | determinística |
| 845 | `adapters::rendering::bar_renderer::tests::bearish_bar_uses_red_color` | determinística |
| 846 | `adapters::rendering::bar_renderer::tests::bullish_bar_uses_green_color` | determinística |
| 847 | `adapters::rendering::bar_renderer::tests::close_tick_is_horizontal` | determinística |
| 848 | `adapters::rendering::bar_renderer::tests::empty_bars_produces_zero_vertices` | determinística |
| 849 | `adapters::rendering::bar_renderer::tests::high_low_is_vertical` | determinística |
| 850 | `adapters::rendering::bar_renderer::tests::open_tick_is_horizontal` | determinística |
| 851 | `adapters::rendering::bar_renderer::tests::tick_width_is_40_percent_of_bar_width` | determinística |
| 852 | `adapters::rendering::bar_renderer::tests::vertex_count_per_bar` | determinística |

---

### fc-input (127 tests)

**Determinísticos:** 127 | **Frágiles:** 0

| # | Test | Clasificación |
|---|------|---------------|
| 853 | `crosshair::tests::clear_position` | determinística |
| 854 | `crosshair::tests::custom_position_override` | determinística |
| 855 | `crosshair::tests::default_mode_is_normal` | determinística |
| 856 | `crosshair::tests::hidden_mode_not_visible` | determinística |
| 857 | `crosshair::tests::magnetic_no_snap_far` | determinística |
| 858 | `crosshair::tests::magnetic_respects_threshold` | determinística |
| 859 | `crosshair::tests::magnetic_snaps_to_data` | determinística |
| 860 | `crosshair::tests::position_snapped_flag` | determinística |
| 861 | `crosshair::tests::set_mode` | determinística |
| 862 | `crosshair::tests::sync_group_different` | determinística |
| 863 | `crosshair::tests::sync_group_none` | determinística |
| 864 | `crosshair::tests::sync_group_same` | determinística |
| 865 | `crosshair::tests::update_position_basic` | determinística |
| 866 | `crosshair::tests::update_position_returns_ref` | determinística |
| 867 | `crosshair::tests::visibility_toggle` | determinística |
| 868 | `engine::tests::arrow_left_produces_pan_negative` | determinística |
| 869 | `engine::tests::arrow_right_produces_pan` | determinística |
| 870 | `engine::tests::arrow_up_produces_pan_price` | determinística |
| 871 | `engine::tests::backspace_produces_delete_selected` | determinística |
| 872 | `engine::tests::delete_produces_delete_selected` | determinística |
| 873 | `engine::tests::double_click_is_noop` | determinística |
| 874 | `engine::tests::drag_cycle_mouse_down_move_up` | determinística |
| 875 | `engine::tests::draw_select_drag_emits_move_selected` | determinística |
| 876 | `engine::tests::escape_cancels_drawing_and_deselects` | determinística |
| 877 | `engine::tests::escape_without_tool_only_deselects` | determinística |
| 878 | `engine::tests::key_1_starts_trend_line` | determinística |
| 879 | `engine::tests::key_9_starts_fibonacci_retracement` | determinística |
| 880 | `engine::tests::minus_key_zooms_out` | determinística |
| 881 | `engine::tests::mouse_down_and_up_cycle` | determinística |
| 882 | `engine::tests::mouse_down_right_button_is_noop` | determinística |
| 883 | `engine::tests::mouse_down_with_drawing_tool_emits_place_point` | determinística |
| 884 | `engine::tests::mouse_down_with_select_tool_emits_select_drawing` | determinística |
| 885 | `engine::tests::mouse_move_produces_update_crosshair` | determinística |
| 886 | `engine::tests::pinch_produces_zoom` | determinística |
| 887 | `engine::tests::plus_key_zooms_in` | determinística |
| 888 | `engine::tests::set_tool_active_tool_is_drawing` | determinística |
| 889 | `engine::tests::shift_tracking_across_key_down_up` | determinística |
| 890 | `engine::tests::stylus_move_produces_crosshair` | determinística |
| 891 | `engine::tests::touch_start_and_end_cycle` | determinística |
| 892 | `engine::tests::wheel_produces_zoom_in` | determinística |
| 893 | `engine::tests::wheel_produces_zoom_out` | determinística |
| 894 | `engine::tests::wheel_with_shift_produces_pan` | determinística |
| 895 | `gesture::tests::config_getter` | determinística |
| 896 | `gesture::tests::default_config` | determinística |
| 897 | `gesture::tests::double_tap` | determinística |
| 898 | `gesture::tests::double_tap_too_slow` | determinística |
| 899 | `gesture::tests::flick_direction` | determinística |
| 900 | `gesture::tests::flick_fast` | determinística |
| 901 | `gesture::tests::long_press` | determinística |
| 902 | `gesture::tests::long_press_with_movement` | determinística |
| 903 | `gesture::tests::multiple_pan_movements` | determinística |
| 904 | `gesture::tests::pan_end_without_flick` | determinística |
| 905 | `gesture::tests::pan_not_started` | determinística |
| 906 | `gesture::tests::pan_start` | determinística |
| 907 | `gesture::tests::pinch_compress` | determinística |
| 908 | `gesture::tests::pinch_spread` | determinística |
| 909 | `gesture::tests::pinch_two_fingers` | determinística |
| 910 | `gesture::tests::reset_clears` | determinística |
| 911 | `gesture::tests::single_tap` | determinística |
| 912 | `gesture::tests::tap_too_far` | determinística |
| 913 | `gesture::tests::tap_too_long` | determinística |
| 914 | `gesture::tests::touch_end_unknown_id_returns_none` | determinística |
| 915 | `gesture::tests::touch_move_unknown_id_returns_none` | determinística |
| 916 | `keyboard::tests::clear_shortcuts` | determinística |
| 917 | `keyboard::tests::default_preset_has_basics` | determinística |
| 918 | `keyboard::tests::description_field` | determinística |
| 919 | `keyboard::tests::duplicate_key_register` | determinística |
| 920 | `keyboard::tests::empty_map` | determinística |
| 921 | `keyboard::tests::find_shortcut` | determinística |
| 922 | `keyboard::tests::handle_event_match` | determinística |
| 923 | `keyboard::tests::handle_event_no_match` | determinística |
| 924 | `keyboard::tests::handle_event_with_modifier` | determinística |
| 925 | `keyboard::tests::handle_event_wrong_modifier` | determinística |
| 926 | `keyboard::tests::len_and_empty` | determinística |
| 927 | `keyboard::tests::minimal_preset_has_few` | determinística |
| 928 | `keyboard::tests::register_shortcut` | determinística |
| 929 | `keyboard::tests::remove_shortcut` | determinística |
| 930 | `keyboard::tests::trading_preset_has_more` | determinística |
| 931 | `pan::tests::auto_scroll_disabled_no_shift` | determinística |
| 932 | `pan::tests::auto_scroll_on_new_data` | determinística |
| 933 | `pan::tests::end_drag_slow` | determinística |
| 934 | `pan::tests::end_drag_with_velocity` | determinística |
| 935 | `pan::tests::follow_price_active` | determinística |
| 936 | `pan::tests::follow_price_inactive` | determinística |
| 937 | `pan::tests::friction_high_stops_fast` | determinística |
| 938 | `pan::tests::friction_low_stops_slow` | determinística |
| 939 | `pan::tests::is_dragging_state` | determinística |
| 940 | `pan::tests::momentum_decelerates` | determinística |
| 941 | `pan::tests::momentum_stops` | determinística |
| 942 | `pan::tests::pan_by_basic` | determinística |
| 943 | `pan::tests::pan_preserves_range` | determinística |
| 944 | `pan::tests::start_drag` | determinística |
| 945 | `pan::tests::update_drag_returns_delta` | determinística |
| 946 | `tests::input_event_clone` | determinística |
| 947 | `tests::input_event_double_click` | determinística |
| 948 | `tests::input_event_key_down` | determinística |
| 949 | `tests::input_event_key_with_modifiers` | determinística |
| 950 | `tests::input_event_mouse_left_click` | determinística |
| 951 | `tests::input_event_mouse_move` | determinística |
| 952 | `tests::input_event_pinch` | determinística |
| 953 | `tests::input_event_stylus` | determinística |
| 954 | `tests::input_event_touch` | determinística |
| 955 | `tests::input_event_wheel` | determinística |
| 956 | `tests::input_event_with_modifiers` | determinística |
| 957 | `tests::key_code_clone_and_hash` | determinística |
| 958 | `tests::modifier_state_default` | determinística |
| 959 | `tests::modifier_state_equality` | determinística |
| 960 | `tests::mouse_button_variants` | determinística |
| 961 | `tests::wheel_with_modifiers` | determinística |
| 962 | `zoom::tests::animated_zoom_completes` | determinística |
| 963 | `zoom::tests::axis_zoom_price` | determinística |
| 964 | `zoom::tests::axis_zoom_time` | determinística |
| 965 | `zoom::tests::box_zoom` | determinística |
| 966 | `zoom::tests::box_zoom_inverted_rect` | determinística |
| 967 | `zoom::tests::pinch_zoom_in` | determinística |
| 968 | `zoom::tests::pinch_zoom_out` | determinística |
| 969 | `zoom::tests::programmatic_zoom` | determinística |
| 970 | `zoom::tests::programmatic_zoom_clamps_too_small` | determinística |
| 971 | `zoom::tests::viewport_width_height` | determinística |
| 972 | `zoom::tests::wheel_zoom_at_left_edge` | determinística |
| 973 | `zoom::tests::wheel_zoom_at_right_edge` | determinística |
| 974 | `zoom::tests::wheel_zoom_clamps_price_when_too_narrow` | determinística |
| 975 | `zoom::tests::wheel_zoom_in` | determinística |
| 976 | `zoom::tests::wheel_zoom_out` | determinística |
| 977 | `zoom::tests::zoom_clamp_max` | determinística |
| 978 | `zoom::tests::zoom_clamp_min` | determinística |
| 979 | `zoom::tests::zoom_level_calculation` | determinística |

---

### fc-primitives (131 tests)

**Determinísticos:** 131 | **Frágiles:** 0

| # | Test | Clasificación |
|---|------|---------------|
| 980 | `bar::tests::bearish_bar` | determinística |
| 981 | `bar::tests::body` | determinística |
| 982 | `bar::tests::body_bearish` | determinística |
| 983 | `bar::tests::bullish_bar` | determinística |
| 984 | `bar::tests::doji_bar` | determinística |
| 985 | `bar::tests::midpoint` | determinística |
| 986 | `bar::tests::range` | determinística |
| 987 | `bar::tests::reject_close_above_high` | determinística |
| 988 | `bar::tests::reject_high_less_than_low` | determinística |
| 989 | `bar::tests::reject_negative_price` | determinística |
| 990 | `bar::tests::reject_open_below_low` | determinística |
| 991 | `bar::tests::valid_construction` | determinística |
| 992 | `color::tests::rgba_blend` | determinística |
| 993 | `color::tests::rgba_default` | determinística |
| 994 | `color::tests::rgba_from_hex` | determinística |
| 995 | `color::tests::rgba_new` | determinística |
| 996 | `color::tests::rgba_rgb` | determinística |
| 997 | `color::tests::rgba_to_f32_array` | determinística |
| 998 | `color::tests::rgba_with_alpha` | determinística |
| 999 | `error::tests::debug_format` | determinística |
| 1000 | `error::tests::display_insufficient_data` | determinística |
| 1001 | `error::tests::display_invalid_price_data` | determinística |
| 1002 | `error::tests::display_out_of_range` | determinística |
| 1003 | `error::tests::implements_std_error` | determinística |
| 1004 | `invalidation::tests::invalidation_clear` | determinística |
| 1005 | `invalidation::tests::invalidation_contains` | determinística |
| 1006 | `invalidation::tests::invalidation_merge_combines_panes` | determinística |
| 1007 | `invalidation::tests::invalidation_merge_takes_higher_level` | determinística |
| 1008 | `invalidation::tests::level_ordering` | determinística |
| 1009 | `invalidation::tests::pane_bitmask_all` | determinística |
| 1010 | `invalidation::tests::pane_bitmask_count` | determinística |
| 1011 | `invalidation::tests::pane_bitmask_iter` | determinística |
| 1012 | `invalidation::tests::pane_bitmask_single` | determinística |
| 1013 | `invalidation::tests::pane_bitmask_union` | determinística |
| 1014 | `kinetic::tests::decay_rate_change` | determinística |
| 1015 | `kinetic::tests::elapsed_tracking` | determinística |
| 1016 | `kinetic::tests::inertia_faster_decay` | determinística |
| 1017 | `kinetic::tests::inertia_mode_basic` | determinística |
| 1018 | `kinetic::tests::kinetic_custom_threshold` | determinística |
| 1019 | `kinetic::tests::kinetic_new` | determinística |
| 1020 | `kinetic::tests::kinetic_start` | determinística |
| 1021 | `kinetic::tests::kinetic_stop` | determinística |
| 1022 | `kinetic::tests::kinetic_stops_at_threshold` | determinística |
| 1023 | `kinetic::tests::kinetic_update_decelerates` | determinística |
| 1024 | `kinetic::tests::momentum_mode_basic` | determinística |
| 1025 | `kinetic::tests::snap_none` | determinística |
| 1026 | `kinetic::tests::snap_target_bar_index` | determinística |
| 1027 | `kinetic::tests::snap_target_price` | determinística |
| 1028 | `kinetic::tests::update_dt_inertia` | determinística |
| 1029 | `kinetic::tests::update_dt_momentum` | determinística |
| 1030 | `kinetic::tests::update_dt_zero_or_negative` | determinística |
| 1031 | `kinetic::tests::with_mode_defaults` | determinística |
| 1032 | `line_style::tests::all_variants_distinct` | determinística |
| 1033 | `line_style::tests::default_is_solid` | determinística |
| 1034 | `localization::tests::english_format_time_label` | determinística |
| 1035 | `localization::tests::english_format_timestamp` | determinística |
| 1036 | `localization::tests::locale_ids` | determinística |
| 1037 | `localization::tests::spanish_format_time_label` | determinística |
| 1038 | `localization::tests::spanish_format_timestamp` | determinística |
| 1039 | `rect::tests::rect_area` | determinística |
| 1040 | `rect::tests::rect_center` | determinística |
| 1041 | `rect::tests::rect_contains` | determinística |
| 1042 | `rect::tests::rect_contains_rect` | determinística |
| 1043 | `rect::tests::rect_full` | determinística |
| 1044 | `rect::tests::rect_intersects` | determinística |
| 1045 | `rect::tests::rect_intersects_or_adjacent` | determinística |
| 1046 | `rect::tests::rect_new` | determinística |
| 1047 | `rect::tests::rect_right_bottom` | determinística |
| 1048 | `rect::tests::rect_to_scissor` | determinística |
| 1049 | `rect::tests::rect_union` | determinística |
| 1050 | `scale::tests::linear_bottom` | determinística |
| 1051 | `scale::tests::linear_equal_min_max` | determinística |
| 1052 | `scale::tests::linear_midpoint` | determinística |
| 1053 | `scale::tests::linear_roundtrip` | determinística |
| 1054 | `scale::tests::linear_top` | determinística |
| 1055 | `scale::tests::scroll_to_end` | determinística |
| 1056 | `scale::tests::scroll_to_end_empty` | determinística |
| 1057 | `scale::tests::scroll_to_end_short_data` | determinística |
| 1058 | `scale::tests::set_bar_spacing` | determinística |
| 1059 | `scale::tests::set_bar_spacing_with_offset` | determinística |
| 1060 | `scale::tests::time_end` | determinística |
| 1061 | `scale::tests::time_equal_start_end` | determinística |
| 1062 | `scale::tests::time_midpoint` | determinística |
| 1063 | `scale::tests::time_roundtrip` | determinística |
| 1064 | `scale::tests::time_start` | determinística |
| 1065 | `scale::tests::visible_bars` | determinística |
| 1066 | `scale::tests::visible_bars_with_offset` | determinística |
| 1067 | `scale::tests::visible_bars_zero_spacing` | determinística |
| 1068 | `scale::tests::visible_range` | determinística |
| 1069 | `scale::tests::visible_range_clamped` | determinística |
| 1070 | `scale::tests::visible_range_empty` | determinística |
| 1071 | `series::tests::default_is_empty` | determinística |
| 1072 | `series::tests::drain_latest_basic` | determinística |
| 1073 | `series::tests::drain_latest_more_than_available` | determinística |
| 1074 | `series::tests::drain_latest_on_empty` | determinística |
| 1075 | `series::tests::drain_returns_newest_first` | determinística |
| 1076 | `series::tests::drop_correctness` | determinística |
| 1077 | `series::tests::empty_series` | determinística |
| 1078 | `series::tests::get_after_wraparound` | determinística |
| 1079 | `series::tests::get_returns_correct_values` | determinística |
| 1080 | `series::tests::get_returns_none_for_out_of_bounds` | determinística |
| 1081 | `series::tests::iter_on_empty` | determinística |
| 1082 | `series::tests::latest_on_empty` | determinística |
| 1083 | `series::tests::many_pushes_after_overflow` | determinística |
| 1084 | `series::tests::overflow_after_wraparound` | determinística |
| 1085 | `series::tests::overflow_overwrites_oldest` | determinística |
| 1086 | `series::tests::push_maintains_order` | determinística |
| 1087 | `series::tests::push_returns_overwritten_after_exact_capacity` | determinística |
| 1088 | `series::tests::push_single` | determinística |
| 1089 | `series_type::tests::all_ten_variants` | determinística |
| 1090 | `series_type::tests::clone_and_eq` | determinística |
| 1091 | `series_type::tests::debug_format` | determinística |
| 1092 | `series_type::tests::default_is_candle` | determinística |
| 1093 | `series_type::tests::display_names` | determinística |
| 1094 | `series_type::tests::is_breakout_pattern` | determinística |
| 1095 | `series_type::tests::is_volume` | determinística |
| 1096 | `series_type::tests::new_variants_are_distinct` | determinística |
| 1097 | `tests::bar_roundtrip` | determinística |
| 1098 | `tests::series_push_and_latest` | determinística |
| 1099 | `tick::tests::reject_ask_less_than_bid` | determinística |
| 1100 | `tick::tests::reject_negative_price` | determinística |
| 1101 | `tick::tests::spread` | determinística |
| 1102 | `tick::tests::valid_construction` | determinística |
| 1103 | `tick::tests::zero_spread` | determinística |
| 1104 | `tick::tests::zero_volume_succeeds` | determinística |
| 1105 | `fc-primitives/src/bar.rs - bar::Bar (line 11)` | determinística |
| 1106 | `fc-primitives/src/color.rs - color::Rgba (line 7)` | determinística |
| 1107 | `fc-primitives/src/invalidation.rs - invalidation::InvalidationMask (line 91)` | determinística |
| 1108 | `fc-primitives/src/kinetic.rs - kinetic::KineticScroll (line 29)` | determinística |
| 1109 | `fc-primitives/src/scale.rs - scale::LinearScale (line 5)` | determinística |
| 1110 | `fc-primitives/src/scale.rs - scale::TimeScale (line 50)` | determinística |

---

### fc-render (145 tests)

**Determinísticos:** 145 | **Frágiles:** 0

| # | Test | Clasificación |
|---|------|---------------|
| 1111 | `backend::tests::backend_is_object_safe` | determinística |
| 1112 | `backend::tests::mock_backend_begin_end_frame` | determinística |
| 1113 | `backend::tests::mock_backend_clip` | determinística |
| 1114 | `backend::tests::mock_backend_default_scale_factor` | determinística |
| 1115 | `backend::tests::mock_backend_executes_commands` | determinística |
| 1116 | `backend::tests::mock_backend_resize` | determinística |
| 1117 | `commands::tests::dashed_line_uses_dashed_style` | determinística |
| 1118 | `commands::tests::display_format` | determinística |
| 1119 | `commands::tests::draw_command_clone` | determinística |
| 1120 | `commands::tests::draw_command_debug` | determinística |
| 1121 | `commands::tests::filled_circle_has_fill_no_stroke` | determinística |
| 1122 | `commands::tests::filled_polygon_is_closed_with_fill` | determinística |
| 1123 | `commands::tests::filled_rect_has_fill_no_stroke` | determinística |
| 1124 | `commands::tests::filled_triangle_has_fill_no_stroke` | determinística |
| 1125 | `commands::tests::line_creates_draw_line` | determinística |
| 1126 | `commands::tests::line_style_default_is_solid` | determinística |
| 1127 | `commands::tests::line_style_equality` | determinística |
| 1128 | `commands::tests::path_clone_preserves_points` | determinística |
| 1129 | `commands::tests::polyline_is_open_no_fill` | determinística |
| 1130 | `commands::tests::stroked_rect_has_stroke_no_fill` | determinística |
| 1131 | `commands::tests::text_label` | determinística |
| 1132 | `commands::tests::z_index_accessor` | determinística |
| 1133 | `context::tests::render_context_clone` | determinística |
| 1134 | `context::tests::render_context_from_pipeline` | determinística |
| 1135 | `context::tests::render_context_new` | determinística |
| 1136 | `coordinates::tests::multiple_roundtrips_stable` | determinística |
| 1137 | `coordinates::tests::narrow_area` | determinística |
| 1138 | `coordinates::tests::offset_area_price` | determinística |
| 1139 | `coordinates::tests::offset_area_timestamp` | determinística |
| 1140 | `coordinates::tests::pipeline_clone` | determinística |
| 1141 | `coordinates::tests::price_above_range` | determinística |
| 1142 | `coordinates::tests::price_below_range` | determinística |
| 1143 | `coordinates::tests::price_to_y_high` | determinística |
| 1144 | `coordinates::tests::price_to_y_low` | determinística |
| 1145 | `coordinates::tests::price_to_y_mid` | determinística |
| 1146 | `coordinates::tests::roundtrip_corners` | determinística |
| 1147 | `coordinates::tests::roundtrip_world_screen_world` | determinística |
| 1148 | `coordinates::tests::scale_factor_2x` | determinística |
| 1149 | `coordinates::tests::scale_factor_3x` | determinística |
| 1150 | `coordinates::tests::screen_point_debug` | determinística |
| 1151 | `coordinates::tests::screen_point_display` | determinística |
| 1152 | `coordinates::tests::timestamp_after_range` | determinística |
| 1153 | `coordinates::tests::timestamp_before_range` | determinística |
| 1154 | `coordinates::tests::timestamp_to_x_max` | determinística |
| 1155 | `coordinates::tests::timestamp_to_x_mid` | determinística |
| 1156 | `coordinates::tests::timestamp_to_x_min` | determinística |
| 1157 | `coordinates::tests::world_point_debug` | determinística |
| 1158 | `coordinates::tests::world_point_display` | determinística |
| 1159 | `coordinates::tests::world_to_screen_and_back` | determinística |
| 1160 | `coordinates::tests::x_to_timestamp_center` | determinística |
| 1161 | `coordinates::tests::x_to_timestamp_left` | determinística |
| 1162 | `coordinates::tests::x_to_timestamp_right` | determinística |
| 1163 | `coordinates::tests::y_to_price_bottom` | determinística |
| 1164 | `coordinates::tests::y_to_price_center` | determinística |
| 1165 | `coordinates::tests::y_to_price_top` | determinística |
| 1166 | `coordinates::tests::zero_span_price` | determinística |
| 1167 | `coordinates::tests::zero_span_time` | determinística |
| 1168 | `dirty::tests::clear_all` | determinística |
| 1169 | `dirty::tests::clear_pass` | determinística |
| 1170 | `dirty::tests::dirty_tracker_new` | determinística |
| 1171 | `dirty::tests::mark_dirty_multiple` | determinística |
| 1172 | `dirty::tests::mark_dirty_single` | determinística |
| 1173 | `dirty::tests::mark_full_dirty` | determinística |
| 1174 | `dirty::tests::merged_rect_multiple` | determinística |
| 1175 | `dirty::tests::merged_rect_single` | determinística |
| 1176 | `dirty::tests::needs_redraw` | determinística |
| 1177 | `drawing_interaction::tests::cancel_resets_placement` | determinística |
| 1178 | `drawing_interaction::tests::fibonacci_extension_needs_three_clicks` | determinística |
| 1179 | `drawing_interaction::tests::horizontal_line_completes_in_one_click` | determinística |
| 1180 | `drawing_interaction::tests::select_mode_no_placement` | determinística |
| 1181 | `drawing_interaction::tests::switch_mode_cancels_placement` | determinística |
| 1182 | `drawing_interaction::tests::trend_line_needs_two_clicks` | determinística |
| 1183 | `drawing_manager::tests::add_and_remove_trend_line` | determinística |
| 1184 | `drawing_manager::tests::bounds_single` | determinística |
| 1185 | `drawing_manager::tests::combined_bounds` | determinística |
| 1186 | `drawing_manager::tests::hit_test_finds_trend_line` | determinística |
| 1187 | `drawing_manager::tests::hit_test_miss` | determinística |
| 1188 | `drawing_manager::tests::hit_test_returns_none_on_empty` | determinística |
| 1189 | `drawing_manager::tests::move_selected` | determinística |
| 1190 | `drawing_manager::tests::move_selected_drawing` | determinística |
| 1191 | `drawing_manager::tests::new_manager_is_empty` | determinística |
| 1192 | `drawing_manager::tests::render_produces_commands` | determinística |
| 1193 | `drawing_manager::tests::render_single` | determinística |
| 1194 | `drawing_manager::tests::select_and_deselect` | determinística |
| 1195 | `drawing_manager::tests::select_deselect_cycle` | determinística |
| 1196 | `indicator_renderer::tests::indicator_renderer_overlay` | determinística |
| 1197 | `indicator_renderer::tests::indicator_renderer_separate` | determinística |
| 1198 | `indicator_renderer::tests::indicator_renderer_z_index` | determinística |
| 1199 | `indicator_renderer::tests::trait_is_send_sync` | determinística |
| 1200 | `layers::tests::all_layers_have_unique_z_ranges` | determinística |
| 1201 | `layers::tests::debug_format` | determinística |
| 1202 | `layers::tests::default_is_candles` | determinística |
| 1203 | `layers::tests::display_format` | determinística |
| 1204 | `layers::tests::hash_consistency` | determinística |
| 1205 | `layers::tests::layer_count` | determinística |
| 1206 | `layers::tests::layers_are_ordered` | determinística |
| 1207 | `layers::tests::z_mid_is_center` | determinística |
| 1208 | `passes::tests::pass_display` | determinística |
| 1209 | `passes::tests::pass_is_not_skippable` | determinística |
| 1210 | `passes::tests::pass_names` | determinística |
| 1211 | `passes::tests::pass_order` | determinística |
| 1212 | `passes::tests::pass_skippable` | determinística |
| 1213 | `passes::tests::pass_tracker_clear_dirty` | determinística |
| 1214 | `passes::tests::pass_tracker_disable` | determinística |
| 1215 | `passes::tests::pass_tracker_mark_all_dirty` | determinística |
| 1216 | `passes::tests::pass_tracker_mark_dirty` | determinística |
| 1217 | `passes::tests::pass_tracker_new` | determinística |
| 1218 | `passes::tests::pass_tracker_passes_to_execute` | determinística |
| 1219 | `passes::tests::pass_z_range` | determinística |
| 1220 | `pipeline::tests::begin_frame_clears_pending` | determinística |
| 1221 | `pipeline::tests::default_impl` | determinística |
| 1222 | `pipeline::tests::empty_end_frame` | determinística |
| 1223 | `pipeline::tests::end_frame_groups_by_pass` | determinística |
| 1224 | `pipeline::tests::end_frame_sorts_by_z_index` | determinística |
| 1225 | `pipeline::tests::execute_runs_dirty_passes` | determinística |
| 1226 | `pipeline::tests::execute_skips_clean_passes` | determinística |
| 1227 | `pipeline::tests::frame_stats` | determinística |
| 1228 | `pipeline::tests::invalidate_all` | determinística |
| 1229 | `pipeline::tests::new_pipeline` | determinística |
| 1230 | `pipeline::tests::reset_clears_state` | determinística |
| 1231 | `pipeline::tests::submit_multiple_commands` | determinística |
| 1232 | `pipeline::tests::submit_single_command` | determinística |
| 1233 | `pipeline::tests::z_index_above_range_goes_to_debug` | determinística |
| 1234 | `pipeline::tests::z_index_negative_goes_to_debug` | determinística |
| 1235 | `pipeline::tests::z_index_to_pass_background` | determinística |
| 1236 | `pipeline::tests::z_index_to_pass_crosshair` | determinística |
| 1237 | `pipeline::tests::z_index_to_pass_series` | determinística |
| 1238 | `pixel_perfect::tests::f32_floor_ceil` | determinística |
| 1239 | `pixel_perfect::tests::f32_snap_midpoint` | determinística |
| 1240 | `pixel_perfect::tests::f32_snap_negative` | determinística |
| 1241 | `pixel_perfect::tests::f32_snap_size_rounds` | determinística |
| 1242 | `pixel_perfect::tests::f32_snap_whole` | determinística |
| 1243 | `pixel_perfect::tests::f64_snap_midpoint` | determinística |
| 1244 | `pixel_perfect::tests::f64_snap_size_rounds` | determinística |
| 1245 | `pixel_perfect::tests::f64_snap_whole` | determinística |
| 1246 | `pixel_perfect::tests::pixel_perfect_rect_outward_snapping` | determinística |
| 1247 | `pixel_perfect::tests::snap_line_exact_one_pixel` | determinística |
| 1248 | `pixel_perfect::tests::snap_line_normal` | determinística |
| 1249 | `pixel_perfect::tests::snap_line_prevents_collapse` | determinística |
| 1250 | `pixel_perfect::tests::snap_point_centres_both_axes` | determinística |
| 1251 | `series_renderer::tests::series_hit_different_index` | determinística |
| 1252 | `series_renderer::tests::series_hit_equality` | determinística |
| 1253 | `fc-render/src/pixel_perfect.rs - pixel_perfect::PixelPerfect (line 22)` | determinística |
| 1254 | `fc-render/src/pixel_perfect.rs - pixel_perfect::pixel_perfect_rect (line 134)` | determinística |
| 1255 | `fc-render/src/pixel_perfect.rs - pixel_perfect::snap_line (line 164)` | determinística |

---

### fc-renderer-wgpu (92 tests)

**Determinísticos:** 92 | **Frágiles:** 0

| # | Test | Clasificación |
|---|------|---------------|
| 1256 | `renderers::area::tests::fill_polygon_is_closed` | determinística |
| 1257 | `renderers::area::tests::new_renderer` | determinística |
| 1258 | `renderers::area::tests::render_empty_data` | determinística |
| 1259 | `renderers::area::tests::render_produces_commands` | determinística |
| 1260 | `renderers::area::tests::render_single_point_produces_nothing` | determinística |
| 1261 | `renderers::bar::tests::new_renderer` | determinística |
| 1262 | `renderers::bar::tests::render_empty_data` | determinística |
| 1263 | `renderers::bar::tests::render_produces_commands` | determinística |
| 1264 | `renderers::baseline::tests::new_renderer` | determinística |
| 1265 | `renderers::baseline::tests::render_empty_data` | determinística |
| 1266 | `renderers::baseline::tests::render_produces_commands` | determinística |
| 1267 | `renderers::baseline::tests::render_single_point_produces_nothing` | determinística |
| 1268 | `renderers::candle::tests::candle_bearish_color` | determinística |
| 1269 | `renderers::candle::tests::candle_bullish_color` | determinística |
| 1270 | `renderers::candle::tests::new_renderer` | determinística |
| 1271 | `renderers::candle::tests::render_empty_data` | determinística |
| 1272 | `renderers::candle::tests::render_produces_commands` | determinística |
| 1273 | `renderers::candle::tests::two_candles_produce_four_commands` | determinística |
| 1274 | `renderers::histogram::tests::histogram_negative_below_zero` | determinística |
| 1275 | `renderers::histogram::tests::histogram_positive_above_zero` | determinística |
| 1276 | `renderers::histogram::tests::new_renderer` | determinística |
| 1277 | `renderers::histogram::tests::render_empty_data` | determinística |
| 1278 | `renderers::histogram::tests::render_produces_commands` | determinística |
| 1279 | `renderers::line::tests::line_has_correct_points` | determinística |
| 1280 | `renderers::line::tests::new_renderer` | determinística |
| 1281 | `renderers::line::tests::render_empty_data` | determinística |
| 1282 | `renderers::line::tests::render_produces_commands` | determinística |
| 1283 | `renderers::line::tests::render_single_point_produces_nothing` | determinística |
| 1284 | `renderers::text::tests::new_renderer` | determinística |
| 1285 | `renderers::text::tests::render_empty_data` | determinística |
| 1286 | `renderers::text::tests::render_produces_commands` | determinística |
| 1287 | `renderers::text::tests::text_has_correct_content` | determinística |
| 1288 | `scissor::tests::current_wgpu_with_active_scissor` | determinística |
| 1289 | `scissor::tests::pop_empty_stack` | determinística |
| 1290 | `scissor::tests::pop_restores` | determinística |
| 1291 | `scissor::tests::push_multiple_intersects` | determinística |
| 1292 | `scissor::tests::push_multiple_no_overlap` | determinística |
| 1293 | `scissor::tests::push_single` | determinística |
| 1294 | `scissor::tests::reset` | determinística |
| 1295 | `scissor::tests::resize` | determinística |
| 1296 | `scissor::tests::scissor_manager_new` | determinística |
| 1297 | `scissor::tests::scissor_rect_contains_inside` | determinística |
| 1298 | `scissor::tests::scissor_rect_contains_outside` | determinística |
| 1299 | `scissor::tests::scissor_rect_full` | determinística |
| 1300 | `scissor::tests::scissor_rect_intersect_contained` | determinística |
| 1301 | `scissor::tests::scissor_rect_intersect_edge_touching` | determinística |
| 1302 | `scissor::tests::scissor_rect_intersect_non_overlapping` | determinística |
| 1303 | `scissor::tests::scissor_rect_intersect_overlapping` | determinística |
| 1304 | `scissor::tests::scissor_rect_new` | determinística |
| 1305 | `scissor::tests::scissor_rect_to_wgpu_flips_y` | determinística |
| 1306 | `scissor::tests::three_panes_sequential` | determinística |
| 1307 | `vertex_gen::tests::circle_fill_vertices_count` | determinística |
| 1308 | `vertex_gen::tests::circle_stroke_vertices_count` | determinística |
| 1309 | `vertex_gen::tests::generate_sorted_vertices_orders_by_z` | determinística |
| 1310 | `vertex_gen::tests::generate_vertices_draw_line` | determinística |
| 1311 | `vertex_gen::tests::generate_vertices_draw_rect_fill_and_stroke` | determinística |
| 1312 | `vertex_gen::tests::generate_vertices_draw_rect_fill_only` | determinística |
| 1313 | `vertex_gen::tests::generate_vertices_image_skipped` | determinística |
| 1314 | `vertex_gen::tests::generate_vertices_text_skipped` | determinística |
| 1315 | `vertex_gen::tests::line_vertices_count` | determinística |
| 1316 | `vertex_gen::tests::line_vertices_positions` | determinística |
| 1317 | `vertex_gen::tests::line_zero_length_returns_nothing` | determinística |
| 1318 | `vertex_gen::tests::ndc_conversion` | determinística |
| 1319 | `vertex_gen::tests::path_closed_adds_last_segment` | determinística |
| 1320 | `vertex_gen::tests::path_fill_vertices_count` | determinística |
| 1321 | `vertex_gen::tests::path_vertices_count` | determinística |
| 1322 | `vertex_gen::tests::rect_fill_vertices_count` | determinística |
| 1323 | `vertex_gen::tests::rect_stroke_vertices_count` | determinística |
| 1324 | `vertex_gen::tests::triangle_fill_vertices_count` | determinística |
| 1325 | `vertex_gen::tests::triangle_stroke_vertices_count` | determinística |
| 1326 | `vertex_gen::tests::z_index_sort` | determinística |
| 1327 | `cache_stores_and_retrieves` | determinística |
| 1328 | `dirty_region_filters_non_dirty_passes` | determinística |
| 1329 | `dirty_region_tracker_union_and_merge` | determinística |
| 1330 | `dirty_tracker_full_dirty_and_per_pass_clear` | determinística |
| 1331 | `full_chart_render_flow` | determinística |
| 1332 | `multi_pass_render_order` | determinística |
| 1333 | `pipeline_execute_and_stats_consistency` | determinística |
| 1334 | `pipeline_full_frame_execution` | determinística |
| 1335 | `pipeline_multiple_frames_reuse` | determinística |
| 1336 | `pipeline_with_disabled_pass` | determinística |
| 1337 | `pixel_perfect_in_render_flow` | determinística |
| 1338 | `pixel_perfect_line_prevents_blur` | determinística |
| 1339 | `renderpass_z_index_ranges` | determinística |
| 1340 | `resize_invalidates_scissor_and_dirty` | determinística |
| 1341 | `scissor_clipping_in_pipeline` | determinística |
| 1342 | `scissor_nested_panes` | determinística |
| 1343 | `scissor_rect_intersection_and_containment` | determinística |
| 1344 | `screen_rect_utilities` | determinística |
| 1345 | `series_renderer_commands_to_pipeline` | determinística |
| 1346 | `vertex_gen_from_draw_commands` | determinística |
| 1347 | `vertex_pod_layout` | determinística |

---

### fc-sessions (22 tests)

**Determinísticos:** 22 | **Frágiles:** 0

| # | Test | Clasificación |
|---|------|---------------|
| 1348 | `tests::london_times` | determinística |
| 1349 | `tests::session_contains_utc_boundary_close` | determinística |
| 1350 | `tests::session_contains_utc_boundary_open` | determinística |
| 1351 | `tests::session_contains_utc_inside` | determinística |
| 1352 | `tests::session_contains_utc_outside` | determinística |
| 1353 | `tests::session_contains_utc_overnight` | determinística |
| 1354 | `tests::session_duration_afterhours` | determinística |
| 1355 | `tests::session_duration_overnight` | determinística |
| 1356 | `tests::session_duration_regular` | determinística |
| 1357 | `tests::session_line_preserves_dimensions` | determinística |
| 1358 | `tests::session_line_renderer_clips_to_visible` | determinística |
| 1359 | `tests::session_line_renderer_default_color` | determinística |
| 1360 | `tests::session_line_renderer_multiple_sessions` | determinística |
| 1361 | `tests::session_line_renderer_produces_lines` | determinística |
| 1362 | `tests::session_line_renderer_skips_inactive` | determinística |
| 1363 | `tests::session_line_renderer_uses_custom_color` | determinística |
| 1364 | `tests::session_new` | determinística |
| 1365 | `tests::tokyo_times` | determinística |
| 1366 | `tests::us_premarket_times` | determinística |
| 1367 | `tests::us_regular_times` | determinística |
| 1368 | `fc-sessions/src/lib.rs - Session (line 12)` | determinística |
| 1369 | `fc-sessions/src/lib.rs - SessionLineRenderer (line 169)` | determinística |

---

### fc-theme (26 tests)

**Determinísticos:** 26 | **Frágiles:** 0

| # | Test | Clasificación |
|---|------|---------------|
| 1370 | `tests::builder_build_produces_complete_theme` | determinística |
| 1371 | `tests::builder_from_theme` | determinística |
| 1372 | `tests::builder_new_starts_with_dark` | determinística |
| 1373 | `tests::builder_override_string` | determinística |
| 1374 | `tests::builder_override_type_safe` | determinística |
| 1375 | `tests::builder_unknown_token_is_ignored` | determinística |
| 1376 | `tests::dark_and_light_differ` | determinística |
| 1377 | `tests::dark_theme_defaults` | determinística |
| 1378 | `tests::handle_clone_shares_state` | determinística |
| 1379 | `tests::handle_hot_swap_color` | determinística |
| 1380 | `tests::handle_hot_swap_entire_theme` | determinística |
| 1381 | `tests::handle_new_and_read` | determinística |
| 1382 | `tests::handle_write_access` | determinística |
| 1383 | `tests::hot_swap_all_tokens_round_trip` | determinística |
| 1384 | `tests::hot_swap_batch_set` | determinística |
| 1385 | `tests::hot_swap_set_color` | determinística |
| 1386 | `tests::light_theme_defaults` | determinística |
| 1387 | `tests::line_style_variants` | determinística |
| 1388 | `tests::preset_dark` | determinística |
| 1389 | `tests::preset_light` | determinística |
| 1390 | `tests::preset_unknown_falls_back_to_dark` | determinística |
| 1391 | `tests::theme_clone` | determinística |
| 1392 | `fc-theme/src/lib.rs - (line 11)` | determinística |
| 1393 | `fc-theme/src/lib.rs - ChartTheme (line 112)` | determinística |
| 1394 | `fc-theme/src/lib.rs - ChartThemeBuilder (line 397)` | determinística |
| 1395 | `fc-theme/src/lib.rs - ThemeHandle (line 492)` | determinística |

---

## Cómo Ejecutar

### Todos

```bash
cargo test --workspace
```

### Por crate

```bash
cargo test -p fc-animation
cargo test -p fc-app
cargo test -p fc-cache
cargo test -p fc-domain
cargo test -p fc-drawing
cargo test -p fc-examples
cargo test -p fc-input
cargo test -p fc-primitives
cargo test -p fc-render
cargo test -p fc-renderer-wgpu
cargo test -p fc-sessions
cargo test -p fc-theme
```

### Integración

```bash
cargo test -p fc-app --tests
cargo test -p fc-drawing --tests
cargo test -p fc-renderer-wgpu --tests
```

### Doc-tests

```bash
cargo test -p fc-animation --doc
cargo test -p fc-app --doc
cargo test -p fc-domain --doc
cargo test -p fc-primitives --doc
cargo test -p fc-render --doc
cargo test -p fc-sessions --doc
cargo test -p fc-theme --doc
```
