use std::thread;
use std::time::Duration;
use chrono::{Datelike, DateTime, Utc};
use uts_ws1::config::Config;
use uts_ws1::logger::LogWriter;

pub fn main() {
    loop {
        // restarts each new day
        loop_logger_for_day()
    }
}

fn loop_logger_for_day() {
    let config = Config::read();
    let start_date = Utc::now();
    let mut writers = create_loggers(&config, &start_date);

    for writer in &mut writers {
        writer.write_header_if_new();
    }

    loop {
        let timestamp = Utc::now();
        for writer in &mut writers {
            writer.write_data(timestamp);
        }

        thread::sleep(Duration::from_secs(config.log_interval as u64));
        if Utc::now().day() != start_date.day() {
            // it's a new day, time to restart
            return;
        }
    }
}

fn create_loggers(config: &Config, start_date: &DateTime<Utc>) -> Vec<LogWriter> {
    let boards = config.create_boards();

    match &config.log_path {
        Some(log_path) => {
            vec![
                LogWriter::create_file_writer(&log_path, boards.clone(), start_date, false),
                LogWriter::create_file_writer(&log_path, boards.clone(), start_date, true),
            ]
        }
        None => {
            panic!("Set UTS_LOG_PATH to store log output")
        }
    }
}

