use std::thread;
use std::time::Duration;
use chrono::Utc;
use clap::{Parser, Subcommand};
use uts_api::config::Config;
use uts_api::logger::LogWriter;

#[derive(Parser)]
#[command(version, about)]
struct CommandLine {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Status,
    /// Log sensor values to stdout
    Log {
        /// I2C buses to poll, use multiple times to specify multiple buses
        #[arg(short, long, default_values = ["1", "2"])]
        bus: Vec<u8>,

        #[arg(short, long)]
        raw: bool,
    },
    /// Control heater
    Heater {
        #[command(subcommand)]
        command: HeaterCommand,
    },
}

#[derive(Subcommand)]
enum HeaterCommand {
    Off,
    Thermostat,
    Thermocool,
    On
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
        Command::Log { bus, raw } => {
            do_log(bus, *raw);
        },
        Command::Status => {
            do_status();
        },
        Command::Heater { .. } => {
            todo!()
        }
    }
}

fn do_status() {
    let config = Config::read();
    let boards = config.create_boards();
    for board in boards {
        if let Some(data) = board.read_display_data(Utc::now()) {
            println!("board:{} temp:{:0.2} mode:{} target:{:0.2} V:{:0.2}/{:0.2} I:{:0.2}",
                     data.board_id,
                     data.u4.unwrap(),
                     data.heater_mode.unwrap_or(uts_api::heater::HeaterMode::OFF),
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
