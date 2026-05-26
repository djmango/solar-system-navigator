use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

use crate::resources::{
    ActiveScenario, EditorState, MapViewMode, RoutePlanner, SimulationClock, SimulationControl,
    SimulationDiagnostics,
};
use crate::ui::components::{
    DiagnosticsText, HelpText, HudRoot, PlannerText, Slider, SliderHandle, SliderType, ValueText,
};

pub fn spawn_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(10.0)),
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.09, 0.12, 0.88)),
            HudRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Solar System Navigator — 3D"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.95, 1.0)),
            ));
            root.spawn((
                Text::new(""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.75, 0.8, 0.9)),
                DiagnosticsText,
            ));
            root.spawn((
                Text::new(help_lines()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.7, 0.78)),
                HelpText,
            ));

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .with_children(|row| {
                spawn_slider(row, "Speed: 1.0x", SliderType::Speed, 1.0, 0.1, 8.0, 22.0);
                spawn_slider(
                    row,
                    "Substeps: 2",
                    SliderType::TicksPerFrame,
                    2.0,
                    1.0,
                    12.0,
                    18.0,
                );
                spawn_slider(row, "Vel X: 0.0", SliderType::VelX, 0.0, -20.0, 20.0, 100.0);
                spawn_slider(row, "Vel Y: 0.0", SliderType::VelY, 0.0, -20.0, 20.0, 100.0);
                spawn_slider(row, "Vel Z: 0.0", SliderType::VelZ, 0.0, -20.0, 20.0, 100.0);
            });

            root.spawn((
                Text::new("Route planner"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.82, 0.55)),
            ));
            root.spawn((
                Text::new(""),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.72, 0.78, 0.88)),
                PlannerText,
            ));
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .with_children(|row| {
                spawn_slider(row, "Δv prograde: 0.5", SliderType::BurnPrograde, 0.5, -5.0, 5.0, 90.0);
                spawn_slider(row, "Δv normal: 0.0", SliderType::BurnNormal, 0.0, -5.0, 5.0, 90.0);
                spawn_slider(row, "Δv radial: 0.0", SliderType::BurnRadial, 0.0, -5.0, 5.0, 90.0);
                spawn_slider(
                    row,
                    "Burn in: 30s",
                    SliderType::BurnTimeOffset,
                    30.0,
                    1.0,
                    180.0,
                    80.0,
                );
                spawn_slider(
                    row,
                    "Hohmann r: 480",
                    SliderType::HohmannTargetRadius,
                    480.0,
                    80.0,
                    900.0,
                    120.0,
                );
            });
        });
}

fn help_lines() -> String {
    "RMB: orbit | MMB: pan | Scroll: zoom | Space: pause | N: step | R: reset | 1-3: missions | Tab: select | P: probe | M: map | B: burn node | C: clear | H: Hohmann Δv1 | Shift+H: Hohmann nodes | O: SOI auto | V: previews | ,/.: burn timing".to_string()
}

fn spawn_slider(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    slider_type: SliderType,
    initial_value: f32,
    min: f32,
    max: f32,
    initial_handle_pos: f32,
) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|col| {
            col.spawn((
                Text::new(label),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ValueText,
                slider_type,
            ));

            col.spawn((
                Button,
                Node {
                    width: Val::Px(180.0),
                    height: Val::Px(18.0),
                    justify_content: JustifyContent::FlexStart,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.22, 0.28)),
                slider_type,
                Slider {
                    value: initial_value,
                    min,
                    max,
                },
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        width: Val::Px(16.0),
                        height: Val::Px(16.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(initial_handle_pos),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.85, 0.88, 0.95)),
                    SliderHandle,
                ));
            });
        });
}

pub fn update_hud_text(
    active: Res<ActiveScenario>,
    simulation: Res<SimulationControl>,
    diagnostics: Res<SimulationDiagnostics>,
    editor: Res<EditorState>,
    clock: Res<SimulationClock>,
    planner: Res<RoutePlanner>,
    map_mode: Res<MapViewMode>,
    mut diag_text: Query<&mut Text, With<DiagnosticsText>>,
    mut planner_text: Query<&mut Text, (With<PlannerText>, Without<DiagnosticsText>)>,
) {
    let Ok(mut text) = diag_text.single_mut() else {
        return;
    };
    let selected = editor.selected_name.as_deref().unwrap_or("(none)");
    **text = format!(
        "Scenario: {} | Speed: {:.1}x | Substeps: {} | Paused: {} | Bodies: {} | E_total: {:.2} (KE {:.2} + PE {:.2}) | Selected: {} | Probe Δv: {:.1}",
        active.name,
        simulation.speed,
        simulation.ticks_per_frame,
        simulation.paused,
        diagnostics.body_count,
        diagnostics.total_energy,
        diagnostics.kinetic_energy,
        diagnostics.potential_energy,
        selected,
        editor.probe_delta_v,
    );

    if let Ok(mut planner_ui) = planner_text.single_mut() {
        let central = planner.central_body.as_deref().unwrap_or("auto");
        let target = planner.target_body.as_deref().unwrap_or("(none)");
        let nodes: String = if planner.nodes.is_empty() {
            "none".to_string()
        } else {
            planner
                .nodes
                .iter()
                .enumerate()
                .map(|(i, n)| {
                    format!(
                        "#{i} t={:.0}s Δv=({:.2},{:.2},{:.2}){}",
                        n.time,
                        n.prograde,
                        n.normal,
                        n.radial,
                        if n.executed { " ✓" } else { "" }
                    )
                })
                .collect::<Vec<_>>()
                .join(" | ")
        };
        let hohmann = planner
            .last_hohmann
            .map(|h| {
                format!(
                    " | Hohmann r2={:.0} Δv1={:.2} Δv2={:.2} T={:.0}s total={:.2}",
                    h.r2,
                    h.dv_departure,
                    h.dv_arrival,
                    h.transfer_time,
                    h.total_delta_v()
                )
            })
            .unwrap_or_default();
        **planner_ui = format!(
            "Sim t: {:.1}s | Map: {} | SOI auto: {} | Central: {} | Target: {} | Nodes: {}{}",
            clock.time,
            if map_mode.active { "ON" } else { "off" },
            if planner.soi_auto { "on" } else { "off" },
            central,
            target,
            nodes,
            hohmann,
        );
    }
}

pub fn ui_system(
    mut simulation_control: ResMut<SimulationControl>,
    mut editor: ResMut<EditorState>,
    mut planner: ResMut<RoutePlanner>,
    mut query_set: ParamSet<(
        Query<(
            Entity,
            &Interaction,
            &GlobalTransform,
            &SliderType,
            &mut Slider,
            &mut BackgroundColor,
        )>,
        Query<(&ChildOf, &mut Node, &mut BackgroundColor), With<SliderHandle>>,
    )>,
    mut value_texts: Query<(&mut Text, &SliderType), With<ValueText>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut cursor: MessageReader<CursorMoved>,
) {
    let cursor_position = cursor.read().last().map(|event| event.position);
    let slider_width = 164.0;

    let mut slider_updates = Vec::new();
    let mut slider_states = Vec::new();

    {
        let mut sliders = query_set.p0();
        let mut active_entity = None;

        for (entity, interaction, _, _, _, _) in &sliders {
            if *interaction == Interaction::Pressed {
                active_entity = Some(entity);
                break;
            }
        }

        for (entity, interaction, transform, slider_type, mut slider, mut background_color) in
            &mut sliders
        {
            let is_active =
                active_entity == Some(entity) && mouse_button.pressed(MouseButton::Left);
            if is_active && let Some(cursor_pos) = cursor_position {
                let node_position = transform.translation().truncate();
                let start_x = node_position.x - slider_width / 2.0;
                let relative_x = (cursor_pos.x - start_x).clamp(0.0, slider_width);
                let percentage = relative_x / slider_width;
                let new_value = match *slider_type {
                    SliderType::TicksPerFrame => {
                        (percentage * (slider.max - slider.min) + slider.min).round()
                    }
                    _ => percentage * (slider.max - slider.min) + slider.min,
                };
                if (new_value - slider.value).abs() > f32::EPSILON {
                    slider.value = new_value;
                    slider_updates.push((*slider_type, new_value));
                }
            }

            slider_states.push((
                entity,
                *interaction,
                slider.value,
                slider.min,
                slider.max,
                is_active,
            ));

            *background_color = if is_active {
                BackgroundColor(Color::srgb(0.35, 0.38, 0.45))
            } else if *interaction == Interaction::Hovered {
                BackgroundColor(Color::srgb(0.28, 0.3, 0.36))
            } else {
                BackgroundColor(Color::srgb(0.2, 0.22, 0.28))
            };
        }
    }

    for (parent, mut handle_node, mut handle_color) in query_set.p1().iter_mut() {
        if let Some((_, interaction, value, min, max, is_active)) = slider_states
            .iter()
            .find(|(entity, ..)| *entity == parent.parent())
        {
            let percentage = (value - min) / (max - min);
            handle_node.left = Val::Px(percentage * slider_width);
            *handle_color = if *is_active {
                BackgroundColor(Color::srgb(0.95, 0.96, 1.0))
            } else if *interaction == Interaction::Hovered {
                BackgroundColor(Color::srgb(0.9, 0.92, 0.98))
            } else {
                BackgroundColor(Color::srgb(0.85, 0.88, 0.95))
            };
        }
    }

    for (slider_type, value) in slider_updates {
        match slider_type {
            SliderType::Speed => simulation_control.speed = value,
            SliderType::TicksPerFrame => {
                simulation_control.ticks_per_frame = value as u32;
            }
            SliderType::VelX => {
                editor.velocity_x = value;
                editor.velocity_dirty = true;
            }
            SliderType::VelY => {
                editor.velocity_y = value;
                editor.velocity_dirty = true;
            }
            SliderType::VelZ => {
                editor.velocity_z = value;
                editor.velocity_dirty = true;
            }
            SliderType::BurnPrograde => planner.draft_prograde = value,
            SliderType::BurnNormal => planner.draft_normal = value,
            SliderType::BurnRadial => planner.draft_radial = value,
            SliderType::BurnTimeOffset => planner.default_burn_offset = value,
            SliderType::HohmannTargetRadius => planner.hohmann_target_radius = value,
        }

        for (mut text, text_slider_type) in &mut value_texts {
            if *text_slider_type == slider_type {
                **text = match slider_type {
                    SliderType::Speed => format!("Speed: {:.1}x", value),
                    SliderType::TicksPerFrame => format!("Substeps: {}", value as u32),
                    SliderType::VelX => format!("Vel X: {value:.1}"),
                    SliderType::VelY => format!("Vel Y: {value:.1}"),
                    SliderType::VelZ => format!("Vel Z: {value:.1}"),
                    SliderType::BurnPrograde => format!("Δv prograde: {value:.2}"),
                    SliderType::BurnNormal => format!("Δv normal: {value:.2}"),
                    SliderType::BurnRadial => format!("Δv radial: {value:.2}"),
                    SliderType::BurnTimeOffset => format!("Burn in: {value:.0}s"),
                    SliderType::HohmannTargetRadius => format!("Hohmann r: {value:.0}"),
                };
            }
        }
    }
}
