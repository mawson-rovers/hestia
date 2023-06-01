use std::thread;
use std::time::Duration;
use chrono::Utc;
use clap::{Parser, Subcommand};
use log::error;
use uts_ws1::board::Board;
use uts_ws1::config::Config;
use uts_ws1::heater::HeaterMode;
use uts_ws1::logger::LogWriter;

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
    Log {
        // Log raw ADC values instead of converted temperatures, voltages, etc.
        #[arg(short, long)]
        raw: bool,
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
        target_sensor: String,
    },

    /// Set PWM duty cycle
    Duty {
        /// Board to update
        #[arg(short, long, required = true)]
        board: u8,

        /// duty cycle (0-255)
        duty: u8,
    }
}

#[derive(Subcommand)]
enum HeaterCommand {
    Off,
    Thermostat,
    Thermocool,
    On,
}

// COMMANDS = {
// "log": lambda: run_logger(),
// "test": lambda args: run_tests(args),
// "temp": lambda: run_temps(),
// "heater": lambda args: run_heater(args),
// "power": lambda args: (set_heater_pwm(int(args[0])) if args
// else print("Power level: %d" % get_heater_pwm())),
// "help": lambda: print("Available commands: %s" % list(COMMANDS.keys())),
// }

pub fn main() {
    let cli = CommandLine::parse();
    match &cli.command {
        Some(command) => match command {
            Command::Log { raw } => do_log(*raw),
            Command::Status => do_status(),
            Command::Heater { board, command } => do_heater(*board, command),
            Command::Target { board, temp } => do_target(*board, *temp),
            Command::TargetSensor { board, target_sensor } => do_target_sensor(*board, target_sensor),
            Command::Duty { board, duty } => do_your_duty(*board, *duty),
        },
        None => do_status()
    }
}

fn do_your_duty(board: u8, duty: u8) {
    let board = single_board(board);
    board.write_heater_pwm(duty);
    show_status(board);
}

fn do_target_sensor(board: u8, target_sensor: &String) {
    let board = single_board(board);
    match target_sensor.to_uppercase().as_str() {
        "TH1" => board.write_target_sensor(0),
        "TH2" => board.write_target_sensor(1),
        "TH3" => board.write_target_sensor(2),
        "J7" => board.write_target_sensor(3),
        "J8" => board.write_target_sensor(4),
        _ => error!("Unsupported target sensor: {}", target_sensor),
    }
    show_status(board);
}

fn do_target(board: u8, temp: f32) {
    let board = single_board(board);
    board.write_target_temp(temp);
    show_status(board);
}

fn single_board(board: u8) -> Board {
    let boards = Config {
        i2c_bus: vec![board],
        ..Config::read()
    }.create_boards();
    assert_eq!(boards.len(), 1, "Only one board");
    boards[0].clone()
}

fn do_heater(board_id: u8, command: &HeaterCommand) {
    let all_boards = Config {
        i2c_bus: vec![1, 2],
        ..Config::read()
    }.create_boards();

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
            HeaterCommand::Thermocool => todo!(),
            HeaterCommand::On => this_board.write_heater_mode(HeaterMode::PWM),
        }
    }

    show_status_all(all_boards);
}

fn do_status() {
    let boards = Config::read().create_boards();
    show_status_all(boards);
}

fn show_status_all(boards: Vec<Board>) {
    for board in boards {
        show_status(board);
    }
}

fn show_status(board: Board) {
    if let Some(data) = board.read_display_data(Utc::now()) {
        let target_sensor_temp = board.read_target_sensor_temp()
            .map(|v| format!("{:0.2}", v))
            .unwrap_or(String::from("#err"));
        let heater_mode = data.heater_mode
            .map(|m| m.to_string())
            .unwrap_or(String::from("#err"));
        println!("board:{} temp:{} heater:{} target:{:0.2} sensor:{} duty:{} V:{:0.2}/{:0.2} I:{:0.2}",
                 data.board_id,
                 target_sensor_temp,
                 heater_mode,
                 data.target_temp.unwrap(),
                 board.get_target_sensor().map(|s| s.id).unwrap_or("#err"),
                 board.read_heater_pwm().unwrap(),
                 data.heater_v_high.unwrap(),
                 data.heater_v_low.unwrap(),
                 data.heater_curr.unwrap());
    } else {
        println!("board:{} #err", board.bus);
    }
}

fn do_log(raw: bool) {
    let config = Config::read();
    let boards = config.create_boards();

    let mut writer = LogWriter::create_stdout_writer(boards, raw);
    writer.write_header_if_new();
    loop {
        let timestamp = Utc::now();
        writer.write_data(timestamp);
        thread::sleep(Duration::from_secs(config.log_interval as u64));
    }
}
