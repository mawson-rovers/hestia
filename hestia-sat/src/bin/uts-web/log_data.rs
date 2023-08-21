use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::zip;
use std::path::PathBuf;

use chrono::{Local, TimeZone, Utc};
use log::{debug, info, warn};
use serde::Serialize;

use uts_ws1::board;
use uts_ws1::payload::Config;
use uts_ws1::csv::TIMESTAMP_FORMAT;

use crate::data::{SystemTimeTempData, TimeTempData};
use crate::status;

pub fn read_logs(config: &Config) -> SystemTimeTempData {
    if let Some(reader) = open_last_log_file(config.log_path.as_ref()) {
        process_file(reader)
    } else {
        warn!("No recent log data to return");
        SystemTimeTempData::new()
    }
}

fn process_file(reader: BufReader<File>) -> SystemTimeTempData {
    let mut lines_iter = reader.lines().map(|l|
        l.unwrap_or_else(|_| String::from("")));
    let headers: Vec<String> = match lines_iter.next() {
        None => {
            warn!("Couldn't read header line from CSV file");
            return SystemTimeTempData::new();
        }
        Some(line) => {
            line.split(",").map(|s| s.to_string()).collect()
        }
    };
    let sensor_whitelist = sensors_to_include();
    let mut result = SystemTimeTempData::new();
    info!("Starting processing lines");
    for (index, line) in zip(2.., lines_iter) {
        process_line(index, line, &headers, &sensor_whitelist, &mut result);
    }
    info!("Finished processing lines");
    result
}

fn process_line(index: usize, line: String, headers: &Vec<String>,
                sensor_whitelist: &HashSet<String>, result: &mut SystemTimeTempData) {
    let values: Vec<&str> = line.split(",").collect();
    debug!("Read CSV values: {:?}", values);

    let timestamp = match Utc.datetime_from_str(values[0], TIMESTAMP_FORMAT) {
        Ok(value) => value,
        Err(_) => {
            // don't print the bogus data, in case it corrupts the log file too
            warn!("Ignoring line {} with invalid timestamp", index);
            return
        }
    };
    let timestamp = Local.from_utc_datetime(&timestamp.naive_local());

    // pull out some additional data for calculated fields
    let (mut v_high, mut v_low, mut curr) = (None::<f32>, None::<f32>, None::<f32>);
    let (mut mode, mut duty) = (None::<&str>, None::<u16>);

    let board_id = values[1];
    for (sensor_id, &value) in zip(&headers[2..], &values[2..]) {
        if sensor_whitelist.contains(sensor_id) && value.len() > 0 {
            result.add(board_id, sensor_id,
                       TimeTempData::new(timestamp, value));
        }
        if sensor_id == "heater_v_high" { v_high = value.parse().ok() }
        if sensor_id == "heater_v_low" { v_low = value.parse().ok() }
        if sensor_id == "heater_curr" { curr = value.parse().ok() }
        if sensor_id == "heater_mode" { mode = Some(value) }
        if sensor_id == "heater_duty" { duty = value.parse().ok() }
    }

    // calculate heater power and add it too
    match (v_high, v_low, curr) {
        (Some(v_high), Some(v_low), Some(curr)) => {
            let power = status::heater_power(v_high, v_low, curr);
            result.add(board_id, "heater_power",
                       TimeTempData::new_f32(timestamp, power));
        }
        _ => {}
    }

    // add derived value for heater_duty between 0.0 and 1.0, depending on mode
    match (mode, duty) {
        (Some("PID"), Some(duty)) => {
            result.add(board_id, "heater_duty",
                       TimeTempData::new_f32(timestamp, f32::from(duty) / 1000.0));
        },
        (_, Some(duty)) => {
            result.add(board_id, "heater_duty",
                       TimeTempData::new_f32(timestamp, f32::from(duty) / 255.0));
        },
        _ => {}
    }
}

fn sensors_to_include() -> HashSet<String> {
    let mut result = HashSet::new();
    for sensor in board::ALL_SENSORS {
        result.insert(sensor.id.into());
    }
    result.insert(String::from("target_temp"));
    result
}

fn open_last_log_file(log_path: Option<&String>) -> Option<BufReader<File>> {
    let log_file = get_last_log_file(log_path?)?;
    info!("Opening last log file: {}", log_file.display());
    let file = File::open(log_file).ok()?;
    Some(BufReader::new(file))
}

fn get_last_log_file(log_path: &String) -> Option<PathBuf> {
    let pattern = format!("{}/uts-data-*[0-9].csv", log_path);
    let mut files: Vec<PathBuf> = glob::glob(pattern.as_str())
        .expect("pattern error")
        .map(Result::unwrap)
        .collect();
    files.sort();
    files.last().cloned()
}

#[derive(Debug, Clone, Serialize)]
pub struct LogFile {
    name: String,
    url: String,
}

pub(crate) fn list_logs(config: &Config) -> Vec<LogFile> {
    let pattern = format!("{}/uts-data-*.csv", config.log_path.as_ref().unwrap());
    let mut files: Vec<PathBuf> = glob::glob(pattern.as_str())
        .expect("pattern error")
        .map(Result::unwrap)
        .collect();
    files.sort();
    files.reverse();
    files.iter()
        .map(|f| {
            // Rust's filename handling is awful :(
            let name = f.file_name().unwrap().to_string_lossy().to_string();
            let name = format!("{}{}", hostname_prefix(), name);
            LogFile { name: name.clone(), url: format!("/api/log/{}", name) }
        })
        .collect()
}

fn hostname_prefix() -> String {
    hostname::get().map(|v| format!("{}-", v.to_string_lossy()))
        .unwrap_or(String::from(""))
}

pub(crate) fn get_log_file(config: &Config, name: &String) -> PathBuf {
    let prefix = hostname_prefix();
    let name = name.strip_prefix(prefix.as_str()).unwrap_or(name);
    PathBuf::from(config.log_path.as_ref().unwrap()).join(name)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;
    use std::path::PathBuf;
    use std::time::Instant;
    use log::{debug, info};
    use crate::data::SystemTimeTempData;
    use crate::log_data::{process_file, process_line, sensors_to_include};

    #[test]
    fn test_process_file() {
        let last = Instant::now();
        let _ = env_logger::try_init();
        let path = PathBuf::from("../var/logs/uts-data-2023-08-16-raw.csv");
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        info!("test setup took {} µs", Instant::now().duration_since(last).as_micros());
        let last = Instant::now();

        let result = process_file(reader);
        info!("process_file took {} µs for {} records",
            Instant::now().duration_since(last).as_micros(),
            result.0.front().unwrap().1.0.front().unwrap().1.len());
        // 21 Aug: process_file took 501057 µs for 1500 records
        let last = Instant::now();

        let json = serde_json::to_string_pretty(&result).unwrap();
        info!("json took {} µs", Instant::now().duration_since(last).as_micros());
        // 21 Aug: json took 353179 µs
        let last = Instant::now();

        assert_eq!(&json[0..512], "{}", "");
        info!("assert_eq took {} µs", Instant::now().duration_since(last).as_micros());
    }

    // #[test]
    fn test_process_line() {
        let last = Instant::now();
        let _ = env_logger::try_init();
        let headers = "UTC,board,TH1,TH2,TH3,U4,U5,U6,U7,TH4,TH5,TH6,J7,J8,J12,J13,J14,J15,J16,\
            heater_v_high,heater_v_low,heater_curr,heater_mode,target_temp,target_sensor,heater_duty";
        let headers: Vec<String> = headers.split(",").map(|s| String::from(s)).collect();
        let line = String::from("2023-08-17 04:17:15.406834,2,24.57,25.74,24.76,25.16,24.63,25.14,\
            25.01,24.64,25.34,25.10,24.52,,,,,,,5.03,5.03,0.01,OFF,0.86,TH1,255");
        let sensors = sensors_to_include();
        let mut result = SystemTimeTempData::new();

        debug!("test setup took {} µs", Instant::now().duration_since(last).as_micros());
        let last = Instant::now();

        process_line(25, line, &headers, &sensors, &mut result);
        debug!("processing line took {} µs", Instant::now().duration_since(last).as_micros());
        let last = Instant::now();

        let json = serde_json::to_string_pretty(&result).unwrap();
        debug!("json took {} µs", Instant::now().duration_since(last).as_micros());
        let last = Instant::now();

        assert_eq!(json, "{}");
        debug!("assert_eq took {} µs", Instant::now().duration_since(last).as_micros());
    }
}
