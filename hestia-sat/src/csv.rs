use std::io;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use chrono::{DateTime, Utc};

pub enum LineEnding {
    LF,
    CRLF,
}

impl LineEnding {
    pub fn to_str(&self) -> &'static str {
        match self {
            LineEnding::LF => "\n",
            LineEnding::CRLF => "\r\n",
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum CsvData {
    F32 {
        value: f32,
    },
    U16 {
        value: u16,
    },
    Timestamp {
        value: DateTime<Utc>,
    },
}

impl From<CsvData> for String {
    fn from(value: CsvData) -> Self {
        match value {
            CsvData::F32 { value } => format!("{:0.4}", value),
            CsvData::U16 { value } => format!("{}", value),
            CsvData::Timestamp { value } => format!("{}", value.format("%Y-%m-%d %T.%6f")),
        }
    }
}

impl From<f32> for CsvData {
    fn from(value: f32) -> Self {
        CsvData::F32 { value }
    }
}

impl From<u16> for CsvData {
    fn from(value: u16) -> Self {
        CsvData::U16 { value }
    }
}

impl From<DateTime<Utc>> for CsvData {
    fn from(value: DateTime<Utc>) -> Self {
        CsvData::Timestamp { value }
    }
}

pub const CSV_FIELD_COUNT: usize = 26;

pub const CSV_HEADERS: [&'static str; CSV_FIELD_COUNT] = [
    "timestamp",
    "board",
    "TH1",
    "TH2",
    "TH3",
    "U4",
    "U5",
    "U6",
    "U7",
    "TH4",
    "TH5",
    "TH6",
    "J7",
    "J8",
    "J12",
    "J13",
    "J14",
    "J15",
    "J16",
    "heater_mode",
    "target_temp",
    "target_sensor",
    "pwm_freq",
    "heater_v_high",
    "heater_v_low",
    "heater_curr",
];

pub struct CsvWriter {
    writer: Box<dyn Write>,
    line_ending: LineEnding,
}

impl CsvWriter {
    pub fn stdout() -> CsvWriter {
        CsvWriter { writer: Box::new(io::stdout()), line_ending: LineEnding::LF }
    }

    pub fn file<P: AsRef<Path>>(path: P) -> io::Result<CsvWriter> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        Ok(CsvWriter { writer: Box::new(file), line_ending: LineEnding::CRLF })
    }

    pub fn write_headers(&mut self) -> io::Result<()> {
        self.write_line(CSV_HEADERS.map(|s| s.to_string()))
    }

    pub fn write_data(&mut self, data: [CsvData; CSV_FIELD_COUNT]) -> io::Result<()> {
        self.write_line(data.map(|d| d.into()))
    }

    fn write_line(&mut self, line: [String; CSV_FIELD_COUNT]) -> io::Result<()>
    {
        let mut write_delim = false;
        for val in line {
            if write_delim {
                self.writer.write_all(b",")?;
            }
            write!(self.writer, "{}", val)?;
            write_delim = true;
        }
        self.writer.write_all(self.line_ending.to_str().as_bytes())?;
        self.writer.flush()?;
        Ok(())
    }
}
