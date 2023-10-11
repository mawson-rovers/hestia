use std::thread;
use std::time::Duration;
use chrono::{Datelike, Utc};
use log::info;
use uts_ws1::payload::{Config, Payload};
use uts_ws1::logger::LogWriter;

pub fn main() {
    let config = Config::read();
    loop {
        // restarts each new day
        loop_logger_for_day(&config);
    }
}

fn loop_logger_for_day(config: &Config) {
    let start_date = Utc::now();
    let log_path = config.log_path.as_ref().expect("Set UTS_LOG_PATH to store log output");
    let payload = Payload::from_config(config);
    info!("Configured with {} boards: {:?}", payload.iter().len(), payload.iter());

    let mut writer = LogWriter::create_file_writer(log_path, &payload, &start_date);
    writer.write_header_if_new();

    loop {
        let timestamp = Utc::now();
        writer.write_data(timestamp);

        thread::sleep(Duration::from_secs(config.log_interval as u64));
        if Utc::now().day() != start_date.day() {
            // it's a new day, time to restart
            return;
        }
    }
}

