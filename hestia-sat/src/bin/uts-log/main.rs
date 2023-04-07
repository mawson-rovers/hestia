use std::{env, io};
use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use chrono::{Datelike, DateTime, Utc};
use csv::{Terminator, WriterBuilder};

use uts_api::board::Board;
use uts_api::{I2C_BUS2, I2cBus};

const LOG_PATH_ENV_VAR: &'static str = "UTS_LOG_PATH";

pub fn main() {
    loop {
        // restarts each new day
        logger()
    }
}

fn logger() {
    let bus = I2C_BUS2;
    let board = match Path::new(bus.path).exists() {
        true => Board::init(&bus),
        false => todo!("Implement stub board"),
    };

    let start_date = Utc::now();
    let (writer, write_header) = get_log_writer(&bus, &start_date);

    let mut writer = WriterBuilder::new()
        .terminator(Terminator::CRLF)
        .from_writer(writer);

    if write_header {
        let mut headers: VecDeque<String> = VecDeque::from(
            board.sensors.iter().map(|s| s.id.to_string()).collect::<Vec<_>>()
        );
        headers.push_front(String::from("timestamp"));
        headers.push_back(String::from("heater"));
        writer.write_record(headers)
            .expect("Failed to write header to new CSV file: {}");
        writer.flush().expect("Failed to flush header output");
    }

    loop {
        let timestamp = Utc::now().format("%Y-%m-%d %T.%f").to_string();
        let temps = board.read_temps();
        let heater_level = if board.is_heater_enabled() { board.read_heater_pwm().unwrap() } else { 0 };

        let mut fields = VecDeque::from(
            temps.into_iter().map(format_temp).collect::<Vec<_>>()
        );
        fields.push_front(timestamp);
        fields.push_back(heater_level.to_string());
        writer.write_record(fields)
            .unwrap_or_else(|e| eprint!("Failed to write to log file: {:?}", e));
        writer.flush().unwrap_or_else(|e| eprint!("Failed to flush header output: {:?}", e));


        sleep(Duration::from_secs(5));
        if Utc::now().day() != start_date.day() {
            return;
        }
    }
}

fn get_log_writer(bus: &I2cBus, start_date: &DateTime<Utc>) -> (Box<dyn io::Write>, bool) {
    match env::var(LOG_PATH_ENV_VAR) {
        Ok(path) => {
            let log_path = Path::new(&path);
            fs::create_dir_all(log_path)
                .expect("Log path does not exist and could not be created: {}");
            let filename = &format!("uts-data-b{}-{}.csv",
                                    bus.id, start_date.format("%Y-%m-%d"));
            let file_path = log_path.join(filename);
            eprintln!("Logging sensor data to {}...", file_path.display());
            let write_header = !file_path.exists();
            (
                Box::new(File::create(&file_path).unwrap()),
                write_header
            )
        }
        Err(_) => {
            (Box::new(io::stdout()), true)
        }
    }
}

fn format_temp(f: f32) -> String {
    match f {
        f if f32::is_nan(f) => String::from(""),
        _ => format!("{:0.2}", f),
    }
}
