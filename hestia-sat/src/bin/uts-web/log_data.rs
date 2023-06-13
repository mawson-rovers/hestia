use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::zip;
use std::path::PathBuf;
use chrono::{TimeZone, Utc};
use log::{debug, info, warn};
use uts_ws1::board;
use uts_ws1::config::Config;
use uts_ws1::csv::TIMESTAMP_FORMAT;
use crate::data::{SystemTimeTempData, TimeTempData};
use crate::status;

pub trait LogReader {
    fn read_logs(&self) -> SystemTimeTempData;
}

impl LogReader for Config {
    fn read_logs(&self) -> SystemTimeTempData {
        let mut result = SystemTimeTempData::new();
        let reader = match open_last_log_file(self.log_path.clone()) {
            None => {
                warn!("No recent log data to return");
                return result;
            }
            Some(r) => r,
        };
        let mut lines_iter = reader.lines().map(|l| l.unwrap());
        let headers: Vec<String> = match lines_iter.next() {
            None => {
                warn!("Couldn't read header line from CSV file");
                return result;
            }
            Some(line) => {
                line.split(",").map(|s| s.to_string()).collect()
            }
        };
        let sensor_whitelist = sensors_to_include();
        debug!("Starting processing lines");
        for line in lines_iter {
            process_line(line, &headers, &sensor_whitelist, &mut result);
        }
        debug!("Finished processing lines");
        result
    }
}

fn process_line(line: String, headers: &Vec<String>, sensor_whitelist: &HashSet<String>, result: &mut SystemTimeTempData) {
    let values: Vec<String> = line.split(",").map(str::to_string).collect();
    debug!("Read CSV values: {:?}", values);
    let timestamp = Utc.datetime_from_str(&values[0], TIMESTAMP_FORMAT).unwrap();
    let board_id = &values[1];
    for (sensor_id, value) in zip(&headers[2..], &values[2..]) {
        if sensor_whitelist.contains(sensor_id) && value.len() > 0 {
            result.add(board_id.clone(), sensor_id.clone(),
                       TimeTempData::new(timestamp, value.clone()));
        }
    }

    // calculate heater power and add it too
    let (mut v_high, mut v_low, mut curr) = (None::<f32>, None::<f32>, None::<f32>);
    for (sensor_id, value) in zip(&headers[2..], &values[2..]) {
        if sensor_id == "heater_v_high" { v_high = value.parse().ok() }
        if sensor_id == "heater_v_low" { v_low = value.parse().ok() }
        if sensor_id == "heater_curr" { curr = value.parse().ok() }
    }
    match (v_high, v_low, curr) {
        (Some(v_high), Some(v_low), Some(curr)) => {
            let power = status::heater_power(v_high, v_low, curr);
            result.add(board_id.clone(), String::from("heater_power"),
                       TimeTempData::new_f32(timestamp, power));
        }
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

fn open_last_log_file(log_path: Option<String>) -> Option<BufReader<File>> {
    let log_file = get_last_log_file(&log_path?)?;
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

