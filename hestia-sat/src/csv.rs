use std::io;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use chrono::{DateTime, Utc};
use crate::board::{DisplayData, SensorData};
use crate::heater::HeaterMode;
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
    Error,
}

impl From<CsvData> for String {
    fn from(value: CsvData) -> Self {
        match value {
            CsvData::F32 { value } => format!("{:0.4}", value),
            CsvData::U16 { value } => format!("{}", value),
            CsvData::Timestamp { value } => format!("{}", value.format("%Y-%m-%d %T.%6f")),
            CsvData::Error => String::from(""), // errors are logged to stderr, not the CSV file
            CsvData::HeaterMode { value } => format!("{}", value),
            CsvData::Sensor { value } => format!("{}", value),
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

impl From<HeaterMode> for CsvData {
    fn from(value: HeaterMode) -> Self {
        CsvData::HeaterMode { value }
    }
}

impl From<Sensor> for CsvData {
    fn from(value: Sensor) -> Self {
        CsvData::Sensor { value }
    }
}

pub const CSV_FIELD_COUNT: usize = 26;

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
    "heater_mode",
    "target_temp",
    "target_sensor",
    "pwm_duty",
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

    //noinspection DuplicatedCode
    pub fn write_sensor_data(&mut self, data: SensorData) {
        self.write_data([
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
    }

    //noinspection DuplicatedCode
    pub fn write_display_data(&mut self, data: DisplayData) {
        self.write_data([
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
