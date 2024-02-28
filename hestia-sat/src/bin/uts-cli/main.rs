use std::thread;
use std::time::Duration;

use chrono::Utc;
use clap::{Parser, Subcommand};
use log::info;

use uts_ws1::board::{Board, BoardDataProvider};
use uts_ws1::heater::{HeaterMode, TargetSensor};
use uts_ws1::logger::LogWriter;
use uts_ws1::payload::{Config, Payload};
use uts_ws1::programs::{Programs, runner};
use uts_ws1::reading::SensorReading;
use uts_ws1::{ReadResult, zipper};

mod test;

#[derive(Parser)]
#[command(version, about)]
struct CommandLine {
    #[command(subcommand)]
    command: Option<Command>,

    /// Provided by flight scheduler and ignored
    #[arg(short)]
    config: Option<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Show status of boards
    ///
    /// Use UTS_I2C_BUS environment variable to configure active boards.
    Status,

    /// Log sensor values to stdout.
    ///
    /// Use UTS_I2C_BUS environment variable to configure active boards.
    Log,

    /// Test board functionality
    ///
    /// Use UTS_I2C_BUS environment variable to configure active boards.
    Test {
        /// Time to run the heater (seconds)
        #[arg(short, long)]
        duration: Option<u8>,
    },

    /// Set heater mode
    Heater {
        /// Board to update. Required if two boards are connected.
        #[arg(short, long)]
        board: Option<u8>,

        /// Mode to configure on the heater. Turning on heater on one board will first
        /// disable it on any other connected boards.
        #[command(subcommand)]
        command: HeaterCommand,
    },

    /// Set target temperature
    Target {
        /// Board to update. Required if two boards are connected.
        #[arg(short, long)]
        board: Option<u8>,

        /// temperature in °C
        temp: f32,
    },

    /// Set target sensor
    TargetSensor {
        /// Board to update. Required if two boards are connected.
        #[arg(short, long)]
        board: Option<u8>,

        /// Target sensor: TH1, TH2, TH3, J7 or J8
        target_sensor: TargetSensor,
    },

    /// Set PWM duty cycle
    Duty {
        /// Board to update. Required if two boards are connected.
        #[arg(short, long)]
        board: Option<u8>,

        /// duty cycle (0-255 for PWM, 0-1000 for PID)
        duty: u16,
    },

    /// Set max heater temperature
    Max {
        /// Board to update. Required if two boards are connected.
        #[arg(short, long)]
        board: Option<u8>,

        /// temperature in °C
        temp: f32,
    },

    /// Run a TOML program file by name
    Run {
        /// Relative or absolute path to TOML file
        toml_file: String,
    },

    /// Compress all the log files in UTS_LOG_PATH
    Zip,

    /// Enable UTS payload on WS-1
    Enable,

    /// Disable UTS payload on WS-1
    Disable,
}

#[derive(Subcommand)]
enum HeaterCommand {
    Off,
    Thermostat,
    On,
}

pub fn main() {
    let cli = CommandLine::parse();
    // don't put any payload logic here - some commands don't require the payload to be on
    match &cli.command {
        Some(command) => match command {
            Command::Log => do_log(),
            Command::Status => do_status(),
            Command::Test { duration } => test::run_test(*duration),
            Command::Heater { board, command } => do_heater(*board, command),
            Command::Target { board, temp } => do_target(*board, *temp),
            Command::TargetSensor { board, target_sensor } => do_target_sensor(*board, *target_sensor),
            Command::Duty { board, duty } => do_duty(*board, *duty),
            Command::Max { board, temp } => do_max(*board, *temp),
            Command::Run { toml_file } => do_run(toml_file),
            Command::Zip => do_zip(),
            Command::Enable => do_enable(),
            Command::Disable => do_disable(),
        },
        None => do_status()
    }
}

fn do_run(toml_file: &str) {
    let payload = Payload::create();
    info!("Configured with {} boards: {:?}", payload.iter().len(), payload.iter());

    let programs = Programs::load_from_file(toml_file);
    info!("Running programs from {}:\n{:#?}", toml_file, programs);

    runner::run(&payload, &programs);
    info!("Programs completed");
}

fn do_duty(board: Option<u8>, duty: u16) {
    update_board(board, |b| b.write_heater_duty(duty));
}

fn do_target_sensor(board: Option<u8>, target_sensor: TargetSensor) {
    update_board(board, |b| b.write_target_sensor(target_sensor));
}

fn do_target(board: Option<u8>, temp: f32) {
    update_board(board, |b| b.write_target_temp(temp));
}

fn do_max(board: Option<u8>, temp: f32) {
    update_board(board, |b| b.write_max_temp(temp));
}

fn update_board<F>(board: Option<u8>, mut op: F)
    where F: FnMut(&Board)
{
    let payload = Payload::create();
    let board = &payload[board];
    op(board);
    show_status(&payload);
}

fn do_zip() {
    let config = Config::read();
    zipper::zip_logs(&config);
}

fn do_enable() {
    let _ = Config::read(); // initialise logger, etc.
    uts_ws1::host::enable_payload();
}

fn do_disable() {
    let _ = Config::read(); // initialise logger, etc.
    uts_ws1::host::disable_payload();
}

fn do_heater(board: Option<u8>, command: &HeaterCommand) {
    let payload = Payload::create();
    let this_board = &payload[board];

    for other in &payload {
        // disable heater on other boards before enabling on this one
        if this_board.id != other.id {
            match command {
                HeaterCommand::Off => {} // do nothing if switching off
                _ => other.write_heater_mode(HeaterMode::OFF),
            }
        }
    }

    // set mode on this board
    match command {
        HeaterCommand::Off => this_board.write_heater_mode(HeaterMode::OFF),
        HeaterCommand::Thermostat => this_board.write_heater_mode(HeaterMode::PID),
        HeaterCommand::On => this_board.write_heater_mode(HeaterMode::PWM),
    }

    show_status(&payload);
}

fn do_status() {
    let payload = Payload::create();
    show_status(&payload);
}

fn show_status(payload: &Payload) {
    for board in payload {
        show_board_status(board);
    }
}

fn format_reading(reading: ReadResult<SensorReading<f32>>) -> String {
    reading.map(|v| format!("{:0.2}", v))
        .unwrap_or(String::from("#err"))
}

fn show_board_status(board: &Board) {
    if let Some(data) = board.read_data() {
        let heater_mode = data.heater_mode
            .map(|m| m.to_string())
            .unwrap_or(String::from("#err"));
        let [.., v_high, v_low, v_curr, _, _, _] = &data.sensors;
        let heater_curr = board.calc_heater_current(v_low.clone(), v_curr.clone());
        println!("board:{} {} temp:{} heater:{} target:{} max:{} sensor:{} duty:{} V:{:0.2}/{:0.2} I:{:0.2} {}",
                 board.bus,
                 board.version,
                 format_reading(board.read_target_sensor_temp()),
                 heater_mode,
                 format_reading(data.target_temp),
                 format_reading(data.max_temp),
                 board.get_target_sensor().map(|s| s.id).unwrap_or("#err"),
                 board.read_heater_duty().unwrap(),
                 v_high.clone().unwrap().display_value,
                 v_low.clone().unwrap().display_value,
                 heater_curr.map_or(String::from("#err"), |c| format!("{:0.2}", c)),
                 data.flags.unwrap(),
        );
    } else {
        println!("board:{} #err", board.bus);
    }
}

fn do_log() {
    let config = Config::read();
    let payload = Payload::create();

    let mut writer = LogWriter::create_stdout_writer(&payload);
    writer.write_header_if_new();
    loop {
        let timestamp = Utc::now();
        writer.write_data(timestamp);
        thread::sleep(Duration::from_secs(config.log_interval as u64));
    }
}
