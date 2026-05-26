use bevy::prelude::*;

#[derive(Component, PartialEq, Copy, Clone)]
pub enum SliderType {
    Speed,
    TicksPerFrame,
    VelX,
    VelY,
    VelZ,
    BurnPrograde,
    BurnNormal,
    BurnRadial,
    BurnTimeOffset,
    HohmannTargetRadius,
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

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct DiagnosticsText;

#[derive(Component)]
pub struct HelpText;

#[derive(Component)]
pub struct PlannerText;
