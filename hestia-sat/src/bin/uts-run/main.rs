use log::info;

use uts_ws1::payload::{Config, Payload};
use uts_ws1::programs::Programs;
use uts_ws1::programs::runner;

pub fn main() {
    let config = Config::read();
    let payload = Payload::from_config(&config);
    info!("Configured with {} boards: {:?}", payload.iter().len(), payload.iter());

    let programs = Programs::load(&config);
    info!("Loaded programs:\n{:#?}", programs);

    runner::run(&payload, &programs);
}
