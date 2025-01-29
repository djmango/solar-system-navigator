use bevy::prelude::*;
use bevy::input::mouse::MouseButton;

use crate::resources::SimulationControl;
use super::components::{Slider, SliderHandle, SliderType, ValueText};

pub fn spawn_ui(commands: &mut Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(50.0),
                padding: UiRect::all(Val::Px(10.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                column_gap: Val::Px(20.0),
                ..default()
            },
            background_color: Color::rgb(0.15, 0.15, 0.15).into(),
            ..default()
        })
        .with_children(|parent| {
            spawn_slider(
                parent,
                "Speed: 1.0x",
                SliderType::Speed,
                1.0,
                0.0,
                5.0,
                36.0,
            );
            spawn_slider(
                parent,
                "Ticks per frame: 1",
                SliderType::TicksPerFrame,
                1.0,
                1.0,
                10.0,
                0.0,
            );
        });
}

fn spawn_slider(
    parent: &mut ChildBuilder,
    label: &str,
    slider_type: SliderType,
    initial_value: f32,
    min: f32,
    max: f32,
    initial_handle_pos: f32,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // Label
            parent.spawn((
                TextBundle::from_section(
                    label,
                    TextStyle {
                        font_size: 20.0,
                        color: Color::WHITE,
                        ..default()
                    },
                ),
                ValueText,
                slider_type,
            ));

            // Slider background
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(20.0),
                            justify_content: JustifyContent::FlexStart,
                            ..default()
                        },
                        background_color: Color::rgb(0.2, 0.2, 0.2).into(),
                        ..default()
                    },
                    slider_type,
                    Slider {
                        value: initial_value,
                        min,
                        max,
                    },
                ))
                .with_children(|parent| {
                    // Slider handle
                    parent.spawn((
                        NodeBundle {
                            style: Style {
                                width: Val::Px(20.0),
                                height: Val::Px(20.0),
                                position_type: PositionType::Absolute,
                                left: Val::Px(initial_handle_pos),
                                ..default()
                            },
                            background_color: Color::rgb(0.8, 0.8, 0.8).into(),
                            ..default()
                        },
                        SliderHandle,
                    ));
                });
        });
}

pub fn ui_system(
    mut simulation_control: ResMut<SimulationControl>,
    mut query_set: ParamSet<(
        Query<(Entity, &Interaction, &Node, &GlobalTransform, &SliderType, &mut Slider, &mut BackgroundColor)>,
        Query<(&Parent, &mut Style, &mut BackgroundColor), With<SliderHandle>>,
    )>,
    mut value_texts: Query<(&mut Text, &SliderType), With<ValueText>>,
    mouse_button: Res<Input<MouseButton>>,
    mut cursor: EventReader<CursorMoved>,
) {
    let cursor_position = cursor.read().last().map(|event| event.position);
    let slider_width = 180.0; // 200px - 20px (handle width)

    // First pass: find active slider and update values
    let mut slider_updates = Vec::new();
    let mut slider_states = Vec::new();
    {
        let mut sliders = query_set.p0();
        let mut active_entity = None;

        // Find the active slider first
        for (entity, interaction, _, _, _, _, _) in sliders.iter() {
            if *interaction == Interaction::Pressed {
                active_entity = Some(entity);
                break;
            }
        }

        // Update sliders and collect changes
        for (entity, interaction, node, transform, slider_type, mut slider, mut background_color) in sliders.iter_mut() {
            let is_active = active_entity.map_or(false, |active| active == entity) && mouse_button.pressed(MouseButton::Left);
            
            if is_active && cursor_position.is_some() {
                let cursor_pos = cursor_position.unwrap();
                let node_position = transform.translation().truncate();
                let start_x = node_position.x - node.size().x / 2.0;
                let relative_x = (cursor_pos.x - start_x).clamp(0.0, slider_width);
                let percentage = relative_x / slider_width;

                let new_value = match *slider_type {
                    SliderType::Speed => percentage * (slider.max - slider.min) + slider.min,
                    SliderType::TicksPerFrame => {
                        (percentage * (slider.max - slider.min) + slider.min).round()
                    }
                };

                if (new_value - slider.value).abs() > f32::EPSILON {
                    slider.value = new_value;
                    slider_updates.push((*slider_type, new_value));
                }
            }

            // Store slider state for handle updates
            slider_states.push((
                entity,
                *interaction,
                slider.value,
                slider.min,
                slider.max,
                is_active,
            ));

            // Update slider background colors
            *background_color = if is_active {
                Color::rgb(0.3, 0.3, 0.3).into()
            } else if *interaction == Interaction::Hovered {
                Color::rgb(0.25, 0.25, 0.25).into()
            } else {
                Color::rgb(0.2, 0.2, 0.2).into()
            };
        }
    }

    // Second pass: update handles using collected states
    for (parent, mut handle_style, mut handle_color) in query_set.p1().iter_mut() {
        if let Some((_, interaction, value, min, max, is_active)) = slider_states
            .iter()
            .find(|(entity, ..)| *entity == parent.get())
        {
            let percentage = (value - min) / (max - min);
            handle_style.left = Val::Px(percentage * slider_width);
            
            *handle_color = if *is_active {
                Color::rgb(0.9, 0.9, 0.9).into()
            } else if *interaction == Interaction::Hovered {
                Color::rgb(0.85, 0.85, 0.85).into()
            } else {
                Color::rgb(0.8, 0.8, 0.8).into()
            };
        }
    }

    // Apply updates to simulation control and text
    for (slider_type, value) in slider_updates {
        match slider_type {
            SliderType::Speed => {
                simulation_control.speed = value;
            }
            SliderType::TicksPerFrame => {
                simulation_control.ticks_per_frame = value as u32;
            }
        }

        // Update corresponding text
        for (mut text, text_slider_type) in value_texts.iter_mut() {
            if *text_slider_type == slider_type {
                text.sections[0].value = match slider_type {
                    SliderType::Speed => format!("Speed: {:.1}x", value),
                    SliderType::TicksPerFrame => format!("Ticks per frame: {}", value as u32),
                };
            }
        }
    }
} 
