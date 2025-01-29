use bevy::prelude::*;

#[derive(Component, PartialEq, Copy, Clone)]
pub enum SliderType {
    Speed,
    TicksPerFrame,
}

#[derive(Component)]
pub struct SliderHandle;

#[derive(Component)]
pub struct Slider {
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct ValueText; 
