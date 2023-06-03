use std::thread;
use std::time::Duration;
use chrono::{Datelike, Utc};
use uts_ws1::config::Config;
use uts_ws1::logger::LogWriter;

pub fn main() {
    loop {
        // restarts each new day
        loop_logger_for_day()
    }
}

fn loop_logger_for_day() {
    let start_date = Utc::now();
    let config = Config::read();
    let log_path = config.log_path.clone().expect("Set UTS_LOG_PATH to store log output");

    let mut writer = LogWriter::create_file_writer(
        &log_path, config.create_boards(), &start_date);
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

