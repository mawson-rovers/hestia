use std::thread;
use std::time::Duration;
use chrono::Utc;
use clap::{Parser, Subcommand};
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
    /// Show status of all connected boards
    Status,

    /// Log sensor values to stdout
    Log {
        /// Board to log, use multiple times to specify multiple boards
        #[arg(short, long, default_values = ["1", "2"])]
        board: Vec<u8>,

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

        /// Target sensor ID
        /// 0 = TH1, 1 = TH2, 2 = TH3, 3 = J7, 4 = J8
        /// 5-7 = Voltage and current sensing, don't use these
        target_sensor: u8,
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
            Command::Log { board, raw } => do_log(board, *raw),
            Command::Status => do_status(),
            Command::Heater { board, command } => do_heater(*board, command),
            Command::Target { board, temp } => do_target(*board, *temp),
            Command::TargetSensor { board, target_sensor } => do_target_sensor(*board, *target_sensor),
        },
        None => do_status()
    }
}

fn do_target_sensor(board: u8, target_sensor: u8) {
    let boards = Config {
        i2c_bus: vec![board],
        ..Config::read()
    }.create_boards();
    for board in boards {
        board.write_target_sensor(target_sensor);
    }
}

fn do_target(board: u8, temp: f32) {
    let boards = Config {
        i2c_bus: vec![board],
        ..Config::read()
    }.create_boards();
    for board in boards {
        board.write_target_temp(temp);
    }
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
            HeaterCommand::Off => { }, // do nothing if switching off
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
}

fn do_status() {
    let boards = Config::read().create_boards();
    for board in boards {
        if let Some(data) = board.read_display_data(Utc::now()) {
            println!("board:{} temp:{:0.2} heater:{} target:{:0.2} V:{:0.2}/{:0.2} I:{:0.2}",
                     data.board_id,
                     board.read_target_sensor_temp().unwrap_or(data.u7.unwrap()),
                     data.heater_mode.unwrap_or(uts_ws1::heater::HeaterMode::OFF),
                     data.target_temp.unwrap(),
                     data.heater_v_high.unwrap(),
                     data.heater_v_low.unwrap(),
                     data.heater_curr.unwrap());
        } else {
            println!("board:{} #err", board.bus);
        }
    }
}

fn do_log(bus: &Vec<u8>, raw: bool) {
    let config = Config {
        i2c_bus: bus.to_owned(),
        ..Config::read()
    };
    let boards = config.create_boards();

    let mut writer = LogWriter::create_stdout_writer(boards, raw);
    writer.write_header_if_new();
    loop {
        let timestamp = Utc::now();
        writer.write_data(timestamp);
        thread::sleep(Duration::from_secs(config.log_interval as u64));
    }
}
