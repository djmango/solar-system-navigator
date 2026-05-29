#[derive(Debug, Clone, serde::Serialize)]
pub struct ManeuverNode {
    pub time: f64,
    pub prograde: f64,
    pub normal: f64,
    pub radial: f64,
    pub executed: bool,
}

impl ManeuverNode {
    pub fn delta_v_magnitude(&self) -> f64 {
        (self.prograde * self.prograde + self.normal * self.normal + self.radial * self.radial)
            .sqrt()
    }
}

pub fn reset_maneuver_execution(nodes: &mut [ManeuverNode]) {
    for node in nodes {
        node.executed = false;
    }
}
