use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use crate::board::{Board, CsvDataProvider};
use crate::csv::CsvWriter;

pub struct LogWriter {
    writer: CsvWriter,
    is_new: bool,
    boards: Vec<Board>,
    read_raw: bool,
}

impl LogWriter {
    pub fn create_stdout_writer(boards: Vec<Board>, read_raw: bool) -> LogWriter {
        let writer = CsvWriter::stdout();
        LogWriter { writer, is_new: true, boards, read_raw }
    }

    pub fn create_file_writer(path: &String, boards: Vec<Board>, start_date: &DateTime<Utc>,
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

    pub fn write_data(&mut self, timestamp: DateTime<Utc>) {
        for board in &self.boards {
            if self.read_raw {
                if let Some(data) = board.read_raw_data() {
                    self.writer.write_raw_data(timestamp, board, data);
                }
            } else {
                if let Some(data) = board.read_display_data() {
                    self.writer.write_display_data(timestamp, board, data);
                }
            }
        }
    }
}
