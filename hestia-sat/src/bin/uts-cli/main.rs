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
        /// Board to update
        #[arg(short, long, required = true)]
        board: u8,

        /// Mode to configure on the heater. Turning on heater on one board will first
        /// disable it on any other connected boards.
        #[command(subcommand)]
        command: HeaterCommand,
    },

    /// Set target temperature
    Target {
        /// Board to update
        #[arg(short, long, required = true)]
        board: u8,

        /// temperature in °C
        temp: f32,
    },

    /// Set target sensor
    TargetSensor {
        /// Board to update
        #[arg(short, long, required = true)]
        board: u8,

        /// Target sensor: TH1, TH2, TH3, J7 or J8
        target_sensor: TargetSensor,
    },

    /// Set PWM duty cycle
    Duty {
        /// Board to update
        #[arg(short, long, required = true)]
        board: u8,

        /// duty cycle (0-255 for PWM, 0-1000 for PID)
        duty: u16,
    },

    /// Set max heater temperature
    Max {
        /// Board to update
        #[arg(short, long, required = true)]
        board: u8,

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
}

#[derive(Subcommand)]
enum HeaterCommand {
    Off,
    Thermostat,
    On,
}

pub fn main() {
    let cli = CommandLine::parse();
    match &cli.command {
        Some(command) => match command {
            Command::Log => do_log(),
            Command::Status => do_status(),
            Command::Test { duration } => test::run_test(*duration),
            Command::Heater { board, command } => do_heater(*board, command),
            Command::Target { board, temp } => do_target(*board, *temp),
            Command::TargetSensor { board, target_sensor } => do_target_sensor(*board, *target_sensor),
            Command::Duty { board, duty } => do_your_duty(*board, *duty),
            Command::Max { board, temp } => do_max(*board, *temp),
            Command::Run { toml_file } => do_run(toml_file),
            Command::Zip => do_zip(),
        },
        None => do_status()
    }
}

fn do_max(board: u8, temp: f32) {
    let board = single_board(board);
    board.write_max_temp(temp);
    show_status(board);
}

fn do_run(toml_file: &str) {
    let payload = Payload::create();
    info!("Configured with {} boards: {:?}", payload.iter().len(), payload.iter());

    let programs = Programs::load_from_file(toml_file);
    info!("Running programs from {}:\n{:#?}", toml_file, programs);

    runner::run(&payload, &programs);
    info!("Programs completed");
}

fn do_your_duty(board: u8, duty: u16) {
    let board = single_board(board);
    board.write_heater_duty(duty);
    show_status(board);
}

fn do_target_sensor(board: u8, target_sensor: TargetSensor) {
    let board = single_board(board);
    board.write_target_sensor(target_sensor);
    show_status(board);
}

fn do_target(board: u8, temp: f32) {
    let board = single_board(board);
    board.write_target_temp(temp);
    show_status(board);
}

fn do_zip() {
    let config = Config::read();
    zipper::zip_logs(&config);
}

fn single_board(board: u8) -> Board {
    Payload::single_board(board).into_board()
}

fn do_heater(board_id: u8, command: &HeaterCommand) {
    let all_boards = Payload::all_boards();

    // disable heater on other boards before enabling on this one
    let other_boards = all_boards.iter().filter(|b| b.bus.id != board_id);
    for other in other_boards {
        match command {
            HeaterCommand::Off => {} // do nothing if switching off
            _ => other.write_heater_mode(HeaterMode::OFF),
        }
    }
    // set mode on this board
    let this_board = all_boards.iter().find(|b| b.bus.id == board_id);
    if let Some(this_board) = this_board {
        match command {
            HeaterCommand::Off => this_board.write_heater_mode(HeaterMode::OFF),
            HeaterCommand::Thermostat => this_board.write_heater_mode(HeaterMode::PID),
            HeaterCommand::On => this_board.write_heater_mode(HeaterMode::PWM),
        }
    }

    show_status_all(all_boards);
}

fn do_status() {
    let payload = Payload::create();
    show_status_all(payload);
}

fn show_status_all(payload: Payload) {
    for board in payload {
        show_status(board);
    }
}

fn format_reading(reading: ReadResult<SensorReading<f32>>) -> String {
    reading.map(|v| format!("{:0.2}", v))
        .unwrap_or(String::from("#err"))
}

fn show_status(board: Board) {
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
    let payload = Payload::from_config(&config);

    let mut writer = LogWriter::create_stdout_writer(&payload);
    writer.write_header_if_new();
    loop {
        let timestamp = Utc::now();
        writer.write_data(timestamp);
        thread::sleep(Duration::from_secs(config.log_interval as u64));
    }
}
