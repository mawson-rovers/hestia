use std::collections::VecDeque;
use std::fs;
use std::fs::{File, OpenOptions};
use std::path::Path;
use std::time::Duration;

use chrono::{Datelike, DateTime, Utc};
use csv::{Terminator, Writer, WriterBuilder};

use uts_api::board::Board;
use uts_api::ReadResult;
use uts_api::config::Config;

pub fn main() {
    env_logger::init();

    loop {
        // restarts each new day
        logger(&Config::read());
    }
}

fn logger(config: &Config) {
    let boards = config.create_boards();

    let start_date = Utc::now();

    let mut writers = LogWriter::create_all(&config, &boards, &start_date);

    for writer in &mut writers {
        writer.write_header_if_new();
    }

    loop {
        let timestamp = Utc::now();
        for writer in &mut writers {
            writer.write_data(timestamp);
        }

        spin_sleep::sleep(Duration::from_secs(5));
        if Utc::now().day() != start_date.day() {
            // it's a new day, it's a new dawn...
            return;
        }
    }
}

struct LogWriter {
    writer: Writer<File>,
    is_new: bool,
    board: Board,
    read_raw: bool,
}

impl LogWriter {
    fn create_all(config: &Config, boards: &Vec<Board>, start_date: &DateTime<Utc>) -> Vec<LogWriter> {
        boards.iter().flat_map(|b| [
            LogWriter::create(&config.log_path, b, start_date, false),
            LogWriter::create(&config.log_path, b, start_date, true),
        ]).collect()
    }

    pub fn create(path: &String, board: &Board, start_date: &DateTime<Utc>, read_raw: bool) -> LogWriter {
        let log_path = Path::new(&path);
        fs::create_dir_all(log_path)
            .expect("Log path does not exist and could not be created: {}");
        let filename = &format!("uts-data-b{}-{}{}.csv",
                                board.bus.id,
                                start_date.format("%Y-%m-%d"),
                                if read_raw { "-raw" } else { "" });
        let file_path = log_path.join(filename);
        eprintln!("Logging i2c{} {} sensor data to {}...",
                  board.bus.id,
                  if read_raw { "raw" } else { "temp" },
                  file_path.display());
        let is_new = !file_path.exists();
        let writer = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .unwrap();
        let writer = WriterBuilder::new()
            .terminator(Terminator::CRLF)
            .from_writer(writer);

        LogWriter { writer, is_new, board: board.to_owned(), read_raw }
    }

    pub fn write_header_if_new(&mut self) {
        if self.is_new {
            let mut headers: VecDeque<String> = VecDeque::from(
                self.board.sensors.iter().map(|s| s.id.to_string()).collect::<Vec<_>>()
            );
            headers.push_front(String::from("timestamp"));
            headers.push_back(String::from("heater"));
            self.writer.write_record(headers)
                .expect("Failed to write header to new CSV file: {}");
            self.writer.flush().expect("Failed to flush header output");
        }
    }

    fn write_data(&mut self, timestamp: DateTime<Utc>) {
        if !self.board.bus.exists() {
            return;
        }

        let heater_level = if self.board.is_heater_enabled() {
            self.board.read_heater_pwm().unwrap()
        } else {
            0
        };

        let mut fields = if self.read_raw {
            format_values(self.board.read_raws())
        } else {
            format_values(self.board.read_temps())
        };
        fields.push_front(timestamp.format("%Y-%m-%d %T.%6f").to_string());
        fields.push_back(heater_level.to_string());

        self.writer.write_record(fields)
            .unwrap_or_else(|e| eprint!("Failed to write to log file: {:?}", e));
        self.writer.flush().unwrap_or_else(|e| eprint!("Failed to flush log output: {:?}", e));
    }
}

trait LogOutput {
    fn format(self) -> String
        where
            Self: Sized;
}

impl LogOutput for ReadResult<f32> {
    fn format(self) -> String {
        match self {
            Ok(f) => format!("{:0.4}", f),
            Err(_) => String::from(""),
        }
    }
}

impl LogOutput for ReadResult<u16> {
    fn format(self) -> String {
        match self {
            Ok(value) => format!("{}", value),
            Err(_) => String::from(""),
        }
    }
}

fn format_values<T: LogOutput>(values: Vec<T>) -> VecDeque<String> {
    VecDeque::from(values.into_iter().map(|v| v.format()).collect::<Vec<_>>())
}
