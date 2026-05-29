use solar_system_navigator::run_app;
use solar_system_navigator::scenario::{load_scenario_relative, scenario_asset_path};

fn main() {
    let initial_path = scenario_asset_path("scenarios/default.toml");
    let template = match load_scenario_relative("scenarios/default.toml") {
        Ok(t) => t,
        Err(err) => {
            eprintln!("Fatal: could not load default scenario at {initial_path}: {err}");
            std::process::exit(1);
        }
    };
    run_app(template);
}
