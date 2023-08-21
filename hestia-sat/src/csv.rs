use std::convert::TryInto;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;

use chrono::{DateTime, Utc};
use chrono::format::{StrftimeItems, Item};
use lazy_static::lazy_static;

use crate::board::{Board, BoardData, BoardFlags};
use crate::heater::HeaterMode;
use crate::reading::SensorReading;
use crate::ReadResult;
use crate::sensors::Sensor;

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
    HeaterMode {
        value: HeaterMode,
    },
    Sensor {
        value: Sensor,
    },
    BoardFlags {
        value: BoardFlags,
    },
    Error,
}

pub const TIMESTAMP_FORMAT: &'static str = "%Y-%m-%d %T.%6f";

lazy_static! {
    pub static ref TIMESTAMP_FORMAT_ITEMS: Vec<Item<'static>> =
        StrftimeItems::new(TIMESTAMP_FORMAT).collect();
}

impl From<CsvData> for String {
    fn from(value: CsvData) -> Self {
        match value {
            CsvData::F32 { value } => format!("{:0.2}", value),
            CsvData::U16 { value } => format!("{}", value),
            CsvData::Timestamp { value } => format!("{}", value.format(TIMESTAMP_FORMAT)),
            CsvData::Error => String::from(""), // errors are logged to stderr, not the CSV file
            CsvData::HeaterMode { value } => format!("{}", value),
            CsvData::Sensor { value } => format!("{}", value),
            CsvData::BoardFlags { value } => format!("{}", value),
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

impl From<u8> for CsvData {
    fn from(value: u8) -> Self {
        CsvData::U16 { value: value as u16 }
    }
}

impl From<DateTime<Utc>> for CsvData {
    fn from(value: DateTime<Utc>) -> Self {
        CsvData::Timestamp { value }
    }
}

impl From<HeaterMode> for CsvData {
    fn from(value: HeaterMode) -> Self {
        CsvData::HeaterMode { value }
    }
}

impl From<&Board> for CsvData {
    fn from(board: &Board) -> Self {
        CsvData::U16 { value: board.bus.id as u16 }
    }
}

impl From<Sensor> for CsvData {
    fn from(value: Sensor) -> Self {
        CsvData::Sensor { value }
    }
}

impl From<BoardFlags> for CsvData {
    fn from(value: BoardFlags) -> Self {
        CsvData::BoardFlags { value }
    }
}

impl<T> From<SensorReading<T>> for CsvData
    where CsvData: From<T>, T: std::fmt::Display {
    fn from(value: SensorReading<T>) -> Self {
        CsvData::from(value.display_value)
    }
}

pub const CSV_FIELD_COUNT: usize = 28;

pub const CSV_HEADERS: [&'static str; CSV_FIELD_COUNT] = [
    "UTC",
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
    "heater_v_high",
    "heater_v_low",
    "heater_curr",
    "heater_mode",
    "target_temp",
    "target_sensor",
    "heater_duty",
    "max_temp",
    "flags",
];

pub struct CsvWriter
{
    open_writer: Box<dyn FnMut() -> io::Result<Box<dyn Write>>>,
    line_ending: LineEnding,
    write_headers: bool,
}

impl CsvWriter {
    pub fn stdout() -> CsvWriter {
        CsvWriter {
            open_writer: Box::new(|| Ok(Box::new(io::stdout()))),
            line_ending: LineEnding::LF,
            write_headers: true,
        }
    }

    pub fn file<P: AsRef<Path> + 'static>(path: P, is_new: bool) -> CsvWriter {
        let options = OpenOptions::new()
            .create(true)
            .append(true)
            .clone();
        CsvWriter {
            open_writer: Box::new(move || {
                let file = options.open(&path)?;
                Ok(Box::new(file))
            }),
            line_ending: LineEnding::CRLF,
            write_headers: is_new,
        }
    }

    pub fn write_headers(&mut self) {
        if self.write_headers {
            self.write_line(CSV_HEADERS.map(|s| s.to_string()))
                .expect("Failed to write header to new CSV file");
        }
    }

    pub fn write_raw_data(&mut self, timestamp: DateTime<Utc>, board: &Board,
                          raw_data: &[ReadResult<u16>; CSV_FIELD_COUNT - 2]) {
        let mut data: Vec<CsvData> = vec![timestamp.into(), board.into()];
        data.extend(raw_data.iter().map(|d| CsvData::from(d)));
        let data: [CsvData; CSV_FIELD_COUNT] = data.try_into().expect("Array sizes didn't match");
        self.write_data(data).unwrap_or_else(|e| eprint!("Failed to write to log file: {:?}", e));
    }

    pub fn write_display_data(&mut self, timestamp: DateTime<Utc>, board: &Board,
                              board_data: &BoardData) {
        let mut data: Vec<CsvData> = vec![timestamp.into(), board.into()];
        data.extend(board_data.sensors.iter().map(|d| CsvData::from(d)));
        data.extend(Some(CsvData::from(&board_data.heater_mode)));
        data.extend(Some(CsvData::from(&board_data.target_temp)));
        data.extend(Some(CsvData::from(&board_data.target_sensor)));
        data.extend(Some(CsvData::from(&board_data.heater_duty)));
        data.extend(Some(CsvData::from(&board_data.max_temp)));
        data.extend(Some(CsvData::from(&board_data.flags)));
        let data: [CsvData; CSV_FIELD_COUNT] = data.try_into().expect("Array sizes didn't match");
        self.write_data(data).unwrap_or_else(|e| eprint!("Failed to write to log file: {:?}", e));
    }

    pub fn write_data(&mut self, data: [CsvData; CSV_FIELD_COUNT]) -> io::Result<()> {
        self.write_line(data.map(|d| d.into()))
    }

    fn write_line(&mut self, line: [String; CSV_FIELD_COUNT]) -> io::Result<()>
    {
        let mut write_delim = false;
        let mut writer = (self.open_writer)()?;
        for val in line {
            if write_delim {
                writer.write_all(b",")?;
            }
            write!(writer, "{}", val)?;
            write_delim = true;
        }
        writer.write_all(self.line_ending.to_str().as_bytes())?;
        writer.flush()
    }
}
