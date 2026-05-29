use sim_core::Simulation;
use sim_core::transfer::HohmannTransfer;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct WasmSimulation {
    inner: Simulation,
}

#[wasm_bindgen]
impl WasmSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmSimulation, JsValue> {
        Simulation::from_default_scenario()
            .map(|inner| Self { inner })
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn from_scenario_toml(toml: &str) -> Result<WasmSimulation, JsValue> {
        Simulation::from_scenario_toml(toml)
            .map(|inner| Self { inner })
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }

    pub fn sim_time(&self) -> f64 {
        self.inner.sim_time
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.inner.paused = paused;
    }

    pub fn paused(&self) -> bool {
        self.inner.paused
    }

    pub fn set_speed(&mut self, speed: f64) {
        self.inner.speed = speed;
    }

    pub fn speed(&self) -> f64 {
        self.inner.speed
    }

    pub fn step(&mut self, real_dt: f64) {
        self.inner.step(real_dt);
    }

    pub fn step_once(&mut self) {
        self.inner.step_once = true;
        self.inner.step(1.0 / 60.0);
    }

    pub fn state_buffer(&self) -> Vec<f64> {
        self.inner.state_buffer()
    }

    pub fn body_snapshots(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.body_snapshots()).map_err(|e| e.into())
    }

    pub fn truth_path_flat(&self, name: &str) -> Vec<f64> {
        self.inner.truth_path_flat(name)
    }

    pub fn predict_maneuver_path(&self) -> Vec<f64> {
        self.inner.predict_maneuver_path()
    }

    pub fn orbit_preview_flat(&self, body_name: &str, segments: u32) -> Vec<f64> {
        self.inner.orbit_preview_flat(body_name, segments as usize)
    }

    pub fn set_target_body(&mut self, name: &str) {
        self.inner.planner.target_body = Some(name.to_string());
    }

    pub fn set_central_body(&mut self, name: &str) {
        self.inner.planner.central_body = Some(name.to_string());
    }

    pub fn set_draft_delta_v(&mut self, prograde: f64, normal: f64, radial: f64) {
        self.inner.planner.draft_prograde = prograde;
        self.inner.planner.draft_normal = normal;
        self.inner.planner.draft_radial = radial;
    }

    pub fn add_maneuver_node(&mut self) {
        self.inner.add_maneuver_node();
    }

    pub fn add_maneuver_node_at_world_position(
        &mut self,
        x: f64,
        y: f64,
        z: f64,
    ) -> Result<f64, JsValue> {
        self.inner
            .add_maneuver_node_at_world_position(x, y, z)
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn predict_maneuver_path_timed(&self) -> Vec<f64> {
        self.inner.predict_maneuver_path_timed()
    }

    pub fn target_orbit_timed_flat(&self, segments: u32) -> Vec<f64> {
        self.inner.target_orbit_timed_flat(segments as usize)
    }

    pub fn maneuver_node_markers_flat(&self) -> Vec<f64> {
        self.inner.maneuver_node_markers_flat()
    }

    pub fn preview_step(&self) -> f64 {
        self.inner.planner.preview_step
    }

    pub fn clear_maneuver_nodes(&mut self) {
        self.inner.clear_maneuver_nodes();
    }

    pub fn update_maneuver_node(
        &mut self,
        index: u32,
        prograde: f64,
        normal: f64,
        radial: f64,
    ) -> Result<(), JsValue> {
        self.inner
            .update_maneuver_node(index as usize, prograde, normal, radial)
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn remove_maneuver_node(&mut self, index: u32) -> Result<(), JsValue> {
        self.inner
            .remove_maneuver_node(index as usize)
            .map_err(|e| JsValue::from_str(&e))
    }

    pub fn set_default_burn_offset(&mut self, offset: f64) {
        self.inner.set_default_burn_offset(offset);
    }

    pub fn set_show_previews(&mut self, show: bool) {
        self.inner.set_show_previews(show);
    }

    pub fn set_soi_auto(&mut self, auto: bool) {
        self.inner.set_soi_auto(auto);
    }

    pub fn planner_state(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.planner).map_err(|e| e.into())
    }

    pub fn diagnostics(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.diagnostics).map_err(|e| e.into())
    }

    pub fn compute_hohmann(&mut self) -> Result<JsValue, JsValue> {
        match self.inner.compute_hohmann() {
            Some(xfer) => serde_wasm_bindgen::to_value(&xfer).map_err(|e| e.into()),
            None => Err(JsValue::from_str("Could not compute Hohmann transfer")),
        }
    }

    pub fn apply_hohmann_departure_draft(&mut self) -> Result<(), JsValue> {
        let xfer: HohmannTransfer = self
            .inner
            .planner
            .last_hohmann
            .ok_or_else(|| JsValue::from_str("No Hohmann computed"))?;
        self.inner.apply_hohmann_departure_draft(&xfer);
        Ok(())
    }

    pub fn add_hohmann_maneuver_pair(&mut self) -> Result<(), JsValue> {
        let xfer = self
            .inner
            .planner
            .last_hohmann
            .ok_or_else(|| JsValue::from_str("No Hohmann computed"))?;
        self.inner.add_hohmann_maneuver_pair(&xfer);
        Ok(())
    }

    pub fn scenario_name(&self) -> String {
        self.inner.scenario.name.clone()
    }
}
