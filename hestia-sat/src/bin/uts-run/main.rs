use std::fmt::Formatter;
use std::thread::sleep;
use std::vec::IntoIter;
use chrono::{DateTime, Duration, Utc};
use log::info;

#[derive(Debug, PartialEq, Clone)]
enum State {
    Heating {
        program: Program,
        end_time: DateTime<Utc>,
    },
    Cooling {
        program: Program,
        end_time: DateTime<Utc>,
    },
    Done,
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone)]
enum Event {
    TemperatureReading {
        temp_sensor: String,
        temp: f32,
    },
    Time,
}

impl State {
    pub fn next(self, event: Event) -> State {
        let current_time = Utc::now();
        match self {
            State::Heating { program, end_time } => match event {
                Event::TemperatureReading { temp_sensor, temp } => {
                    info!("Checking {} <=> {}, {} <=> {}", temp, program.temp_abort,
                        temp_sensor, program.temp_sensor);
                    if temp > program.temp_abort && temp_sensor == program.temp_sensor {
                        info!("Too hot - aborting program: {}", &program);
                        let end_time = current_time + program.cooling_time;
                        State::Cooling { program, end_time }
                    } else {
                        State::Heating { program, end_time }
                    }
                }
                Event::Time => {
                    if current_time >= end_time {
                        info ! ("Outta time - completing program: {}", & program);
                        let end_time = current_time + program.cooling_time;
                        State::Cooling { program, end_time }
                    } else {
                        State::Heating { program, end_time }
                    }
                },
            },
            State::Cooling { program, end_time } => match event {
                Event::Time if current_time >= end_time => {
                    match program.next {
                        Some(program) => {
                            let end_time = current_time + program.heating_time;
                            State::Heating { program: *program, end_time }
                        },
                        None => State::Done,
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
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Heating { end_time, .. } => write!(f, "State::Heating({})", end_time),
            State::Cooling { end_time, .. } => write!(f, "State::Cooling({})", end_time),
            State::Done => write!(f, "State::Done"),
            State::Failed { message } => write!(f, "State::Failed(\"{}\")", message),
        }
    }
}

#[repr(u8)]
#[derive(Debug, PartialEq, Clone)]
enum HeaterPosition {
    Top = 1,
    Bottom = 2,
}

#[derive(Debug, PartialEq, Clone)]
struct Program {
    id: usize,
    heating_time: Duration,
    temp_sensor: String,
    temp_abort: f32,
    thermostat: Option<f32>,
    cooling_time: Duration,
    heater_position: HeaterPosition,
    heater_duty: f32,
    next: Option<Box<Program>>,
}

impl Program {
    fn run(self, events: IntoIter<Event>) {
        let end_time = Utc::now() + self.heating_time;
        let mut state = State::Heating { program: self, end_time };
        for event in events {
            info!("{} <- {:?}", &state, &event);
            if state == State::Done { break }
            state = state.next(event);
            sleep(Duration::seconds(1).to_std().unwrap());
        }
    }
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Program#{}", self.id)
    }
}

pub fn main() {
    env_logger::init();

    let last = Program {
        id: 2,
        heating_time: Duration::seconds(5),
        temp_sensor: String::from("TH1"),
        temp_abort: 80.0,
        thermostat: None,
        cooling_time: Duration::seconds(10),
        heater_position: HeaterPosition::Top,
        heater_duty: 1.0,
        next: None,
    };
    let first = Program {
        id: 1,
        heating_time: Duration::seconds(3),
        temp_sensor: String::from("J7"),
        temp_abort: 100.0,
        thermostat: Some(80.0),
        cooling_time: Duration::seconds(10),
        heater_position: HeaterPosition::Bottom,
        heater_duty: 0.0,
        next: Some(Box::new(last)),
    };

    let events: Vec<Event> = vec![
        Event::Time,
        Event::TemperatureReading { temp: 55.0, temp_sensor: String::from("J7") },
        Event::TemperatureReading { temp: 105.0, temp_sensor: String::from("J7") },
        Event::Time,
        Event::Time,
        Event::TemperatureReading { temp: 60.0, temp_sensor: String::from("J7") },
        Event::TemperatureReading { temp: 80.0, temp_sensor: String::from("TH1") },
        Event::Time,
        Event::Time,
        Event::TemperatureReading { temp: 120.0, temp_sensor: String::from("TH1") },
        Event::TemperatureReading { temp: 100.0, temp_sensor: String::from("TH1") },
        Event::Time,
        Event::Time,
        Event::TemperatureReading { temp: 120.0, temp_sensor: String::from("J7") },
        Event::TemperatureReading { temp: 90.0, temp_sensor: String::from("TH1") },
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

    first.run(events.into_iter());
}