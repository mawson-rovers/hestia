use std::fs;
use std::path::Path;
use std::time::Duration;

use chrono::{Datelike, DateTime, Utc};

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
    writer: CsvWriter,
    is_new: bool,
    board: Board,
    read_raw: bool,
}

impl LogWriter {
    fn create_all(config: &Config, boards: &Vec<Board>, start_date: &DateTime<Utc>) -> Vec<LogWriter> {
        match &config.log_path {
            None => {
                Vec::from([
                    LogWriter::create_stdout_writer(&boards[0])
                ])
            }
            Some(log_path) => {
                boards.iter().flat_map(|b| [
                    LogWriter::create_file_writer(&log_path, b, start_date, false),
                    LogWriter::create_file_writer(&log_path, b, start_date, true),
                ]).collect()
            }
        }
    }

    pub fn create_stdout_writer(board: &Board) -> LogWriter {
        LogWriter {
            writer: CsvWriter::stdout(),
            is_new: true,
            board: board.to_owned(),
            read_raw: true
        }
    }

    pub fn create_file_writer(path: &String, board: &Board, start_date: &DateTime<Utc>,
                              read_raw: bool) -> LogWriter {
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
        let writer = CsvWriter::file(file_path)
            .expect("Unable to create log file");

        LogWriter { writer, is_new, board: board.to_owned(), read_raw }
    }

    pub fn write_header_if_new(&mut self) {
        if self.is_new {
            self.writer.write_headers()
                .expect("Failed to write header to new CSV file");
        }
    }

    fn write_data(&mut self, timestamp: DateTime<Utc>) {
        if !self.board.bus.exists() {
            return;
        }

        let data = self.board.read_sensors(timestamp, self.board.bus.id as u16);
        
        self.writer.write_data([
            data.timestamp.into(),
            data.board_id.into(),
            data.th1.unwrap_or(u16::MAX).into(),
            data.th2.unwrap_or(u16::MAX).into(),
            data.th3.unwrap_or(u16::MAX).into(),
            data.u4.unwrap_or(u16::MAX).into(),
            data.u5.unwrap_or(u16::MAX).into(),
            data.u6.unwrap_or(u16::MAX).into(),
            data.u7.unwrap_or(u16::MAX).into(),
            data.th4.unwrap_or(u16::MAX).into(),
            data.th5.unwrap_or(u16::MAX).into(),
            data.th6.unwrap_or(u16::MAX).into(),
            data.j7.unwrap_or(u16::MAX).into(),
            data.j8.unwrap_or(u16::MAX).into(),
            data.j12.unwrap_or(u16::MAX).into(),
            data.j13.unwrap_or(u16::MAX).into(),
            data.j14.unwrap_or(u16::MAX).into(),
            data.j15.unwrap_or(u16::MAX).into(),
            data.j16.unwrap_or(u16::MAX).into(),
            data.heater_mode.unwrap_or(u16::MAX).into(),
            data.target_temp.unwrap_or(u16::MAX).into(),
            data.target_sensor.unwrap_or(u16::MAX).into(),
            data.pwm_freq.unwrap_or(u16::MAX).into(),
            data.heater_v_high.unwrap_or(u16::MAX).into(),
            data.heater_v_low.unwrap_or(u16::MAX).into(),
            data.heater_curr.unwrap_or(u16::MAX).into(),
        ]).unwrap_or_else(|e| eprint!("Failed to write to log file: {:?}", e));
    }
}
