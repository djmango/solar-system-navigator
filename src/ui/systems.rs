use bevy::input::mouse::MouseButton;
use bevy::prelude::*;

use crate::astro::{self, AU};
use crate::camera::OrbitCamera;
use crate::components::{CelestialBody, Position};
use crate::resources::{
    ActiveScenario, EditorState, GameUx, MapViewMode, RoutePlanner, SimulationClock,
    SimulationControl, SimulationDiagnostics,
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
                spawn_slider(
                    row,
                    "Speed: 50000x",
                    SliderType::Speed,
                    astro::DEFAULT_TIME_WARP,
                    1.0,
                    500_000.0,
                    90.0,
                );
                spawn_slider(
                    row,
                    "Substeps: 4",
                    SliderType::TicksPerFrame,
                    4.0,
                    1.0,
                    16.0,
                    18.0,
                );
                spawn_slider(
                    row,
                    "Vel X: 0",
                    SliderType::VelX,
                    0.0,
                    -80_000.0,
                    80_000.0,
                    90.0,
                );
                spawn_slider(
                    row,
                    "Vel Y: 0",
                    SliderType::VelY,
                    0.0,
                    -80_000.0,
                    80_000.0,
                    90.0,
                );
                spawn_slider(
                    row,
                    "Vel Z: 0",
                    SliderType::VelZ,
                    0.0,
                    -80_000.0,
                    80_000.0,
                    90.0,
                );
                spawn_slider(
                    row,
                    "Sim t: 0s",
                    SliderType::SimTime,
                    0.0,
                    0.0,
                    astro::YEAR,
                    0.0,
                );
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
                spawn_slider(
                    row,
                    "Δv prograde: 0",
                    SliderType::BurnPrograde,
                    0.0,
                    -5000.0,
                    5000.0,
                    90.0,
                );
                spawn_slider(
                    row,
                    "Δv normal: 0",
                    SliderType::BurnNormal,
                    0.0,
                    -5000.0,
                    5000.0,
                    90.0,
                );
                spawn_slider(
                    row,
                    "Δv radial: 0",
                    SliderType::BurnRadial,
                    0.0,
                    -5000.0,
                    5000.0,
                    90.0,
                );
                spawn_slider(
                    row,
                    "Node lead: 0d",
                    SliderType::BurnTimeOffset,
                    0.0,
                    0.0,
                    760.0,
                    0.0,
                );
                spawn_slider(
                    row,
                    "Hohmann r (AU)",
                    SliderType::HohmannTargetRadius,
                    AU * 1.524,
                    0.3 * AU,
                    5.0 * AU,
                    120.0,
                );
            });
        });
}

fn help_lines() -> String {
    "Flight: LMB orbit · RMB/MMB pan · click select · dbl-click/F frame · Home primary · G follow · Tab cycle · Space pause · M map\n\
     Map planning: click orbit to add node · click node to select · [ ] cycle · X/Del delete · , . slide time · C clear · sliders edit Δv & lead · H/Shift+H Hohmann · O SOI auto"
        .to_string()
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
    game_ux: Res<GameUx>,
    cameras: Query<&OrbitCamera, With<Camera3d>>,
    bodies: Query<(&CelestialBody, &Position)>,
    mut diag_text: Query<&mut Text, With<DiagnosticsText>>,
    mut planner_text: Query<&mut Text, (With<PlannerText>, Without<DiagnosticsText>)>,
) {
    let Ok(mut text) = diag_text.single_mut() else {
        return;
    };
    let selected = editor.selected_name.as_deref().unwrap_or("(none)");
    let view_dist = cameras
        .single()
        .map(|c| astro::format_length(c.radius))
        .unwrap_or_else(|_| "?".to_string());
    let orbit_note = if game_ux.show_system_orbits {
        "N-body truth paths"
    } else {
        "hidden"
    };
    let follow = if editor.follow_selection { "on" } else { "off" };
    **text = format!(
        "Scenario: {} | SI physics | View: {} | Orbits: {orbit_note} | Follow: {follow} | Warp: {:.0}x | Paused: {} | t={:.0}s | Selected: {} | Probe Δv: {:.0} m/s | E: {:.3e} J",
        active.name,
        view_dist,
        simulation.speed,
        simulation.paused,
        clock.time,
        selected,
        editor.probe_delta_v,
        diagnostics.total_energy,
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
                    let frame = n.central_body.as_deref().unwrap_or(central);
                    let lead_d = (n.time - clock.time) / astro::DAY;
                    let sel = if planner.selected_node == Some(n.id) {
                        ">"
                    } else {
                        " "
                    };
                    format!(
                        "{sel}#{i} {frame} lead={lead_d:.1}d Δv={:.0} ({:.0},{:.0},{:.0}){}",
                        n.delta_v_magnitude(),
                        n.prograde,
                        n.normal,
                        n.radial,
                        if n.executed { " ✓" } else { "" }
                    )
                })
                .collect::<Vec<_>>()
                .join("  ")
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
        let map_bodies = if map_mode.active {
            let mut lines: Vec<String> = bodies
                .iter()
                .map(|(b, p)| {
                    let r = p.0.length();
                    format!("{} @ {}", b.name, astro::format_length(r))
                })
                .collect();
            lines.sort();
            format!(" | Bodies: {}", lines.join(", "))
        } else {
            String::new()
        };
        **planner_ui = format!(
            "Map: {} | SOI auto: {} | Central: {} | Target: {} | Nodes: {}{} | Paths = same N-body integrator as sim{map_bodies}",
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
    mut clock: ResMut<SimulationClock>,
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
            SliderType::BurnPrograde => {
                planner.draft_prograde = value;
                if let Some(node) = planner.selected_mut() {
                    node.prograde = value;
                }
            }
            SliderType::BurnNormal => {
                planner.draft_normal = value;
                if let Some(node) = planner.selected_mut() {
                    node.normal = value;
                }
            }
            SliderType::BurnRadial => {
                planner.draft_radial = value;
                if let Some(node) = planner.selected_mut() {
                    node.radial = value;
                }
            }
            SliderType::BurnTimeOffset => {
                planner.default_burn_offset = value * astro::DAY;
                let target_time = clock.time + value * astro::DAY;
                if let Some(node) = planner.selected_mut() {
                    node.time = target_time;
                    planner.nodes.sort_by(|a, b| a.time.total_cmp(&b.time));
                }
            }
            SliderType::HohmannTargetRadius => planner.hohmann_target_radius = value,
            SliderType::SimTime => {
                if simulation_control.paused {
                    clock.time = value;
                }
            }
        }

        for (mut text, text_slider_type) in &mut value_texts {
            if *text_slider_type == slider_type {
                **text = match slider_type {
                    SliderType::Speed => format!("Warp: {:.0}", value),
                    SliderType::TicksPerFrame => format!("Substeps: {}", value as u32),
                    SliderType::VelX => format!("Vel X: {value:.1}"),
                    SliderType::VelY => format!("Vel Y: {value:.1}"),
                    SliderType::VelZ => format!("Vel Z: {value:.1}"),
                    SliderType::BurnPrograde => format!("Δv prograde: {value:.1}"),
                    SliderType::BurnNormal => format!("Δv normal: {value:.1}"),
                    SliderType::BurnRadial => format!("Δv radial: {value:.1}"),
                    SliderType::BurnTimeOffset => format!("Node lead: {value:.1}d"),
                    SliderType::HohmannTargetRadius => {
                        format!("Hohmann r: {:.3} AU", value / AU)
                    }
                    SliderType::SimTime => format!("Sim t: {value:.0}s"),
                };
            }
        }
    }
}

/// Push the selected maneuver node's values into the planner sliders when the
/// selection changes, so the handles and labels reflect the node being edited.
pub fn sync_planner_sliders(
    mut planner: ResMut<RoutePlanner>,
    clock: Res<SimulationClock>,
    mut sliders: Query<(&SliderType, &mut Slider)>,
    mut value_texts: Query<(&mut Text, &SliderType), With<ValueText>>,
) {
    if !planner.sync_sliders {
        return;
    }
    planner.sync_sliders = false;

    let (prograde, normal, radial, lead_days) = match planner.selected() {
        Some(node) => (
            node.prograde,
            node.normal,
            node.radial,
            ((node.time - clock.time) / astro::DAY).max(0.0),
        ),
        None => (
            planner.draft_prograde,
            planner.draft_normal,
            planner.draft_radial,
            0.0,
        ),
    };

    for (slider_type, mut slider) in &mut sliders {
        let v = match slider_type {
            SliderType::BurnPrograde => prograde,
            SliderType::BurnNormal => normal,
            SliderType::BurnRadial => radial,
            SliderType::BurnTimeOffset => lead_days,
            _ => continue,
        };
        slider.value = v.clamp(slider.min, slider.max);
    }

    for (mut text, slider_type) in &mut value_texts {
        **text = match slider_type {
            SliderType::BurnPrograde => format!("Δv prograde: {prograde:.1}"),
            SliderType::BurnNormal => format!("Δv normal: {normal:.1}"),
            SliderType::BurnRadial => format!("Δv radial: {radial:.1}"),
            SliderType::BurnTimeOffset => format!("Node lead: {lead_days:.1}d"),
            _ => continue,
        };
    }
}
