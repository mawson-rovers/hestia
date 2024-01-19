use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use log::info;
use crate::board::BoardDataProvider;
use crate::csv::CsvWriter;
use crate::payload::Payload;

pub struct LogWriter<'a> {
    writer: CsvWriter,
    raw_writer: Option<CsvWriter>,
    payload: &'a Payload,
}

impl<'a> LogWriter<'a> {
    pub fn create_stdout_writer(payload: &'a Payload) -> LogWriter<'a> {
        let writer = CsvWriter::stdout();
        LogWriter { writer, raw_writer: None, payload }
    }

    pub fn create_file_writer(path: &String, payload: &'a Payload, start_date: &DateTime<Utc>, compressed: bool) -> LogWriter<'a> {
        let log_path = Path::new(&path);
        fs::create_dir_all(log_path)
            .expect("Log path does not exist and could not be created: {}");
        let writer = Self::new_csv_writer(start_date, log_path, false, compressed);
        let raw_writer = Self::new_csv_writer(start_date, log_path, true, compressed);

        LogWriter { writer, raw_writer: Some(raw_writer), payload }
    }

    fn new_csv_writer(start_date: &DateTime<Utc>, log_path: &Path, raw_log: bool, compressed: bool) -> CsvWriter {
        let filename = &format!("uts-data-{}{}.csv{}",
                                start_date.format("%Y-%m-%d"),
                                if raw_log { "-raw" } else { "" },
                                if compressed { ".bz2" } else { "" });
        let file_path = log_path.join(filename);
        info!("Logging {} sensor data to {}...",
                  if raw_log { "raw" } else { "temp" },
                  file_path.display());
        let is_new = !file_path.exists();
        if compressed {
            CsvWriter::compressed_file(file_path, is_new)
        } else {
            CsvWriter::file(file_path, is_new)
        }
    }

    pub fn write_header_if_new(&mut self) {
        self.writer.write_display_headers();
        if let Some(raw_writer) = &mut self.raw_writer {
            raw_writer.write_raw_headers();
        }
    }

    pub fn write_data(&mut self, timestamp: DateTime<Utc>) {
        for board in self.payload {
            if let Some(data) = board.read_data() {
                self.writer.write_display_data(timestamp, board, &data);
                if let Some(raw_writer) = &mut self.raw_writer {
                    raw_writer.write_raw_data(timestamp, board, &data.get_raw_data());
                }
            }
        }
    }
}
