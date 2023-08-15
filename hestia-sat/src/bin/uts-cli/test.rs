use std::fmt::Formatter;
use std::iter::zip;
use std::thread;
use std::time::Duration;

use colored::{ColoredString, Colorize};

use uts_ws1::board::{Board, BoardDataProvider, BoardVersion};
use uts_ws1::heater::HeaterMode;
use uts_ws1::payload::Payload;
use uts_ws1::reading::SensorReading;
use uts_ws1::ReadResult;
use uts_ws1::sensors::SensorId;

struct TestData<'a> {
    board: &'a Board,
    target_sensor_temp: f32,
    heater_mode: HeaterMode,
    target_temp: f32,
    target_sensor: SensorId,
    heater_duty: u16,
    heater_voltage: f32,
    heater_curr: f32,
    sensor_readings: [ReadResult<SensorReading<f32>>; 17],
}

impl TestData<'_> {
    fn heater_power(&self) -> f32 {
        self.heater_voltage * self.heater_curr
    }

    fn heater_resistance(&self) -> f32 {
        self.heater_voltage / self.heater_curr
    }
}

impl std::fmt::Display for TestData<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
               "board:{} {} temp:{:.2} heater:{} target:{:.2} max:{:.2} sensor:{} duty:{} \
               P:{:.2} V:{:.2} I:{:.2} R:{:.2}",
               self.board.bus,
               self.board.version,
               self.target_sensor_temp,
               self.heater_mode,
               self.target_temp,
               0.0,
               self.target_sensor,
               self.heater_duty,
               self.heater_power(),
               self.heater_voltage,
               self.heater_curr,
               self.heater_resistance(),
        )
    }
}

/// Ensures the heater is turned off if test aborts
struct BoardTest {
    payload: Payload,
}

impl BoardTest {
    pub fn new() -> Self {
        BoardTest { payload: Payload::create() }
    }
}

impl Drop for BoardTest {
    fn drop(&mut self) {
        // turn off heater even if a test fails
        for board in &self.payload {
            board.write_heater_mode(HeaterMode::OFF);
        }
    }
}

pub fn run_test(duration: Option<u8>) {
    let test = BoardTest::new();
    for board in &test.payload {
        log::info!("Testing board {} on /dev/i2c-{}", board.version, board.bus);
        test_sensors(board);
        test_heater(board, duration);

        log::info!("Testing for board {} complete", board.bus);
    }
}

fn nominal_temperature(reading: SensorReading<f32>) -> bool {
    return reading.display_value > 10.0 &&
        reading.display_value < 40.0;
}

fn color_result(result: bool) -> ColoredString {
    if result {
        "OK".green()
    } else {
        "FAIL".red()
    }
}

fn assert_f32(check: bool, label: &'static str, value: f32) {
    log::info!("{}: {} ({:.2})", label, color_result(check), value);
    assert!(check, "Value out of range: {}", label);
}

fn test_sensors(board: &Board) {
    log::info!("Starting sensor test");
    let data = read_board(board);
    for (sensor, reading) in zip(&board.sensors[0..17], data.sensor_readings) {
        match reading {
            Ok(reading) => {
                let temp_ok = nominal_temperature(reading);
                log::info!("Sensor {}: {} ({:.2})", sensor, color_result(temp_ok), reading);
                assert!(temp_ok, "Reading out of range: {}", reading);
            }
            Err(_) => {
                log::info!("Sensor {}: {}", sensor, "IGNORED".cyan());
            }
        }
    }
}

fn test_heater(board: &Board, duration: Option<u8>) {
    log::info!("Testing heater on board {}", board.bus);

    let data = read_board(board);
    log::info!("{}", data);
    assert_eq!(HeaterMode::OFF, data.heater_mode);
    let start_temp = data.target_sensor_temp;
    assert_f32(start_temp < 80.0, "start_temp", start_temp);
    assert_f32(data.heater_voltage < 0.1, "start_voltage", data.heater_voltage);
    assert_eq!(255, data.heater_duty, "heater_duty");

    log::info!("Turning on heater");
    board.write_heater_mode(HeaterMode::PWM);
    thread::sleep(Duration::from_secs(1));

    let data = read_board(board);
    assert_f32(data.heater_voltage > 4.0, "heater_voltage", data.heater_voltage);
    assert_f32(data.heater_curr > 1.0, "heater_current", data.heater_curr);
    assert_eq!(HeaterMode::PWM, data.heater_mode);

    let duration = duration.unwrap_or(10);
    log::info!("Waiting {} seconds for things to warm up a bit", duration);
    thread::sleep(Duration::from_secs(duration as u64));

    let data = read_board(board);
    log::info!("{}", data);
    let end_temp = data.target_sensor_temp;
    let temp_diff = end_temp - start_temp;
    assert_f32(temp_diff > 5.0, "Temperature difference", temp_diff);
    assert_f32(data.heater_voltage > 4.0, "end_voltage", data.heater_voltage);
    assert_f32(data.heater_curr > 1.0, "end_current", data.heater_curr);

    log::info!("Turning heater off");
    board.write_heater_mode(HeaterMode::OFF);
    let data = read_board(board);
    log::info!("{}", data);
    assert_eq!(HeaterMode::OFF, data.heater_mode);
    assert_f32(data.heater_voltage < 0.1, "off_voltage", data.heater_voltage);
}

fn read_board(board: &Board) -> TestData {
    let data = board.read_data().unwrap_or_else(|| {
        panic!("Failed to read data from board {}", board.bus);
    });
    let target_sensor_temp = board.read_target_sensor_temp().unwrap().display_value;
    let target_temp = data.target_temp.unwrap().display_value;
    let heater_mode = data.heater_mode.unwrap().display_value;
    let target_sensor = data.target_sensor.unwrap().display_value.id;
    let heater_duty = data.heater_duty.unwrap().display_value;
    let [sensor_readings @ .., heater_v_high, heater_v_low, heater_curr] = data.sensors;
    let heater_v_high = heater_v_high.unwrap().display_value;
    let heater_v_low = heater_v_low.unwrap().display_value;
    let heater_curr = heater_curr.unwrap().display_value;
    let (heater_voltage, heater_curr) = match board.version {
        BoardVersion::V1_1 => (0.0, 0.0),
        BoardVersion::V2_0 => (heater_v_high - heater_v_low, heater_curr),
        BoardVersion::V2_2 => (
            heater_v_high - heater_v_low,
            (heater_v_low - heater_curr) / 0.05),
    };
    TestData {
        board,
        target_sensor_temp,
        heater_mode,
        target_temp,
        target_sensor,
        heater_duty,
        heater_voltage,
        heater_curr,
        sensor_readings,
    }
}
