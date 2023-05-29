use std::fs;
use std::path::Path;
use std::time::Duration;

use chrono::{Datelike, DateTime, Utc};
use log::error;

use uts_api::board::Board;
use uts_api::config::Config;
use uts_api::csv::CsvWriter;

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

    let mut writers = LogWriter::create_all(&config, boards, &start_date);

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
    writer: CsvWriter,
    is_new: bool,
    boards: Vec<Board>,
    read_raw: bool,
}

impl LogWriter {
    fn create_all(config: &Config, boards: Vec<Board>, start_date: &DateTime<Utc>) -> Vec<LogWriter> {
        match &config.log_path {
            None => {
                vec![LogWriter::create_stdout_writer(boards)]
            }
            Some(log_path) => {
                vec![
                    LogWriter::create_file_writer(&log_path, boards.clone(), start_date, false),
                    LogWriter::create_file_writer(&log_path, boards.clone(), start_date, true),
                ]
            }
        }
    }

    pub fn create_stdout_writer(boards: Vec<Board>) -> LogWriter {
        LogWriter {
            writer: CsvWriter::stdout(),
            is_new: true,
            boards,
            read_raw: true
        }
    }

    pub fn create_file_writer<'a>(path: &String, boards: Vec<Board>, start_date: &DateTime<Utc>,
                              read_raw: bool) -> LogWriter {
        let log_path = Path::new(&path);
        fs::create_dir_all(log_path)
            .expect("Log path does not exist and could not be created: {}");
        let filename = &format!("uts-data-{}{}.csv",
                                start_date.format("%Y-%m-%d"),
                                if read_raw { "-raw" } else { "" });
        let file_path = log_path.join(filename);
        eprintln!("Logging {} sensor data to {}...",
                  if read_raw { "raw" } else { "temp" },
                  file_path.display());
        let is_new = !file_path.exists();
        let writer = CsvWriter::file(file_path)
            .expect("Unable to create log file");

        LogWriter { writer, is_new, boards, read_raw }
    }

    pub fn write_header_if_new(&mut self) {
        if self.is_new {
            self.writer.write_headers()
                .expect("Failed to write header to new CSV file");
        }
    }

    fn write_data(&mut self, timestamp: DateTime<Utc>) {
        for board in &self.boards {
            LogWriter::write_board_data(&mut self.writer, board, timestamp);
        }
    }

    fn write_board_data(writer: &mut CsvWriter, board: &Board, timestamp: DateTime<Utc>) {
        if !board.bus.exists() {
            return;
        }

        let board_id: u16 = board.bus.id as u16;
        let data = board.read_sensors(timestamp, board_id);
        match data {
            Some(data) => {
                writer.write_data([
                    data.timestamp.into(),
                    data.board_id.into(),
                    data.th1.into(),
                    data.th2.into(),
                    data.th3.into(),
                    data.u4.into(),
                    data.u5.into(),
                    data.u6.into(),
                    data.u7.into(),
                    data.th4.into(),
                    data.th5.into(),
                    data.th6.into(),
                    data.j7.into(),
                    data.j8.into(),
                    data.j12.into(),
                    data.j13.into(),
                    data.j14.into(),
                    data.j15.into(),
                    data.j16.into(),
                    data.heater_mode.into(),
                    data.target_temp.into(),
                    data.target_sensor.into(),
                    data.pwm_freq.into(),
                    data.heater_v_high.into(),
                    data.heater_v_low.into(),
                    data.heater_curr.into(),
                ]).unwrap_or_else(|e| eprint!("Failed to write to log file: {:?}", e));
            },
            None => {
                error!("Failed to read check sensor {} on I2C bus {}", board.check_sensor, board_id)
            }
        }
    }
}
