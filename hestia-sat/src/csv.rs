use std::convert::TryInto;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::path::Path;

use chrono::{DateTime, Utc};
use chrono::format::{Item, StrftimeItems};
use flate2::Compression;
use flate2::write::GzEncoder;
use lazy_static::lazy_static;
use log::error;

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

pub const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %T.%6f";

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

pub const CSV_RAW_FIELD_COUNT: usize = 31;

pub const CSV_RAW_HEADERS: [&str; CSV_RAW_FIELD_COUNT] = [
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
    "v_high",
    "v_low",
    "v_curr",
    "v_high_avg",
    "v_low_avg",
    "v_curr_avg",
    "heater_mode",
    "target_temp",
    "target_sensor",
    "heater_duty",
    "max_temp",
    "flags",
];

pub const CSV_DISPLAY_FIELD_COUNT: usize = 28;

pub const CSV_DISPLAY_HEADERS: [&str; CSV_DISPLAY_FIELD_COUNT] = [
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
    "heater_voltage",
    "heater_curr",
    "heater_power",
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

    pub fn compressed_file<P: AsRef<Path> + 'static>(path: P, is_new: bool) -> CsvWriter {
        let options = OpenOptions::new()
            .create(true)
            .append(true)
            .clone();
        CsvWriter {
            open_writer: Box::new(move || {
                let file = options.open(&path)?;
                let encoder = GzEncoder::new(file, Compression::fast());
                Ok(Box::new(encoder))
            }),
            line_ending: LineEnding::CRLF,
            write_headers: is_new,
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

    pub fn write_raw_headers(&mut self) {
        if self.write_headers {
            self.write_line(CSV_RAW_HEADERS.map(|s| s.to_string()))
                .expect("Failed to write header to new CSV file");
        }
    }

    pub fn write_display_headers(&mut self) {
        if self.write_headers {
            self.write_line(CSV_DISPLAY_HEADERS.map(|s| s.to_string()))
                .expect("Failed to write header to new CSV file");
        }
    }

    pub fn write_raw_data(&mut self, timestamp: DateTime<Utc>, board: &Board,
                          raw_data: &[ReadResult<u16>; CSV_RAW_FIELD_COUNT - 2]) {
        let mut data: Vec<CsvData> = vec![timestamp.into(), board.into()];
        data.extend(raw_data.iter().map(CsvData::from));
        let data: [CsvData; CSV_RAW_FIELD_COUNT] = data.try_into().expect("Array sizes didn't match");
        self.write_data(data).unwrap_or_else(|e| error!("Failed to write to log file: {:?}", e));
    }

    pub fn write_display_data(&mut self, timestamp: DateTime<Utc>, board: &Board,
                              board_data: &BoardData) {
        let [.., v_high_avg, v_low_avg, v_curr_avg] = &board_data.sensors;
        let heater_voltage = board.calc_heater_voltage(v_high_avg.clone(), v_low_avg.clone());
        let heater_curr = board.calc_heater_current(v_low_avg.clone(), v_curr_avg.clone());
        let heater_power = board.calc_heater_power(v_high_avg.clone(), v_low_avg.clone(), v_curr_avg.clone());
        let data: [CsvData; CSV_DISPLAY_FIELD_COUNT] = [
            timestamp.into(),
            board.into(),
            CsvData::from(&board_data.sensors[0]),
            CsvData::from(&board_data.sensors[1]),
            CsvData::from(&board_data.sensors[2]),
            CsvData::from(&board_data.sensors[3]),
            CsvData::from(&board_data.sensors[4]),
            CsvData::from(&board_data.sensors[5]),
            CsvData::from(&board_data.sensors[6]),
            CsvData::from(&board_data.sensors[7]),
            CsvData::from(&board_data.sensors[8]),
            CsvData::from(&board_data.sensors[9]),
            CsvData::from(&board_data.sensors[10]),
            CsvData::from(&board_data.sensors[11]),
            CsvData::from(&board_data.sensors[12]),
            CsvData::from(&board_data.sensors[13]),
            CsvData::from(&board_data.sensors[14]),
            CsvData::from(&board_data.sensors[15]),
            CsvData::from(&board_data.sensors[16]),
            CsvData::from(heater_voltage),
            CsvData::from(heater_curr),
            CsvData::from(heater_power),
            CsvData::from(&board_data.heater_mode),
            CsvData::from(&board_data.target_temp),
            CsvData::from(&board_data.target_sensor),
            CsvData::from(&board_data.heater_duty),
            CsvData::from(&board_data.max_temp),
            CsvData::from(&board_data.flags),
        ];
        self.write_data(data).unwrap_or_else(|e| error!("Failed to write to log file: {:?}", e));
    }

    pub fn write_data<const LEN: usize>(&mut self, data: [CsvData; LEN]) -> io::Result<()> {
        self.write_line(data.map(|d| d.into()))
    }

    fn write_line<const LEN: usize>(&mut self, line: [String; LEN]) -> io::Result<()>
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
