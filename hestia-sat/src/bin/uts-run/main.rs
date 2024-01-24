use chrono::Duration;
use log::info;

use uts_ws1::payload::{Config, Payload};

use uts_ws1::programs::Programs;
use uts_ws1::programs::runner::{PayloadController, PayloadEvents};

pub fn main() {
    let config = Config::read();
    let payload = Payload::from_config(&config);
    info!("Configured with {} boards: {:?}", payload.iter().len(), payload.iter());

    let programs = Programs::load(&config);
    info!("Loaded programs:\n{:#?}", programs);

    loop {
        let mut events = PayloadEvents::new(&payload);
        let program_list = &mut programs.iter();
        let mut controller = PayloadController::new(&payload, program_list);
        controller.run(&mut events, Duration::seconds(1));
        if !programs.run_loop || controller.is_aborted() { break; }
    }
}
