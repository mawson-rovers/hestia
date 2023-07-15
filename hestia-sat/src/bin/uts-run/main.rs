mod config;

use std::fmt::Formatter;
use std::slice::Iter;
use std::thread::sleep;
use chrono::{DateTime, Duration, Utc};
use log::info;
use crate::config::Program;

#[derive(Debug, PartialEq)]
enum State<'a> {
    Heating {
        program: &'a Program,
        end_time: DateTime<Utc>,
    },
    Cooling {
        program: &'a Program,
        end_time: DateTime<Utc>,
    },
    Done,
    Failed {
        message: String,
    },
}

#[derive(Debug)]
enum Event {
    TemperatureReading {
        temp_sensor: String,
        temp: f32,
    },
    Time,
}

impl <'a> State<'a> {
    pub fn next(self, programs: &mut Iter<'a, Program>, event: Event) -> State<'a> {
        let current_time = Utc::now();
        match self {
            State::Heating { program, end_time } => match event {
                Event::TemperatureReading { temp_sensor, temp } => {
                    info!("Checking {} <=> {}, {} <=> {}", temp, program.temp_abort,
                        temp_sensor, program.temp_sensor);
                    if temp > program.temp_abort && temp_sensor == program.temp_sensor {
                        info!("Too hot - aborting program: {:?}", &program);
                        let end_time = current_time + program.cooling_time;
                        State::Cooling { program, end_time }
                    } else {
                        State::Heating { program, end_time }
                    }
                }
                Event::Time => {
                    if current_time >= end_time {
                        info ! ("Outta time - completing program: {:?}", &program);
                        let end_time = current_time + program.cooling_time;
                        State::Cooling { program, end_time }
                    } else {
                        State::Heating { program, end_time }
                    }
                },
            },
            State::Cooling { program, end_time } => match event {
                Event::Time if current_time >= end_time => {
                    if let Some(program) = programs.next() {
                        let end_time = current_time + program.heating_time;
                        State::Heating { program, end_time }
                    } else {
                        State::Done
                    }
                }
                _ => {
                    State::Cooling { program, end_time }
                }
            },
            State::Done => State::Done,
            state => State::Failed {
                message: format!("Invalid event {:#?} for state: {:#?}", event, state)
            }
        }
    }

    pub fn start(programs: &mut Iter<'a, Program>) -> State<'a> {
        let first = programs.next().expect("Didn't find any programs");
        let end_time = Utc::now() + first.heating_time;
        State::Heating { program: first, end_time }
    }
}

impl std::fmt::Display for State<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            State::Heating { end_time, .. } => write!(f, "State::Heating({})", end_time),
            State::Cooling { end_time, .. } => write!(f, "State::Cooling({})", end_time),
            State::Done => write!(f, "State::Done"),
            State::Failed { message } => write!(f, "State::Failed(\"{}\")", message),
        }
    }
}

pub fn main() {
    env_logger::init();
    let config = config::load();
    info!("Loaded config:\n{:#?}", config);

    let events: Vec<Event> = vec![
        Event::Time,
        Event::TemperatureReading { temp: 55.0, temp_sensor: String::from("TH1") },
        Event::TemperatureReading { temp: 105.0, temp_sensor: String::from("TH1") },
        Event::Time,
        Event::Time,
        Event::TemperatureReading { temp: 60.0, temp_sensor: String::from("TH1") },
        Event::TemperatureReading { temp: 80.0, temp_sensor: String::from("J7") },
        Event::Time,
        Event::Time,
        Event::TemperatureReading { temp: 120.0, temp_sensor: String::from("J7") },
        Event::TemperatureReading { temp: 100.0, temp_sensor: String::from("J7") },
        Event::Time,
        Event::Time,
        Event::TemperatureReading { temp: 120.0, temp_sensor: String::from("TH1") },
        Event::TemperatureReading { temp: 90.0, temp_sensor: String::from("J7") },
        Event::Time,
        Event::Time,
        Event::TemperatureReading { temp: 60.0, temp_sensor: String::from("TH1") },
        Event::TemperatureReading { temp: 60.0, temp_sensor: String::from("J7") },
        Event::Time,
        Event::Time,
        Event::Time,
        Event::Time,
        Event::Time,
        Event::Time,
        Event::Time,
        Event::Time,
        Event::Time,
        Event::Time,
        Event::Time,
        Event::Time,
        Event::Time,
    ];

    let mut programs = config.programs();
    let mut state = State::start(&mut programs);
    for event in events {
        info!("{} <- {:?}", &state, &event);
        if state == State::Done { break }
        state = state.next(&mut programs, event);
        sleep(Duration::seconds(1).to_std().unwrap());
    }
}