mod config;

use std::fmt::Formatter;
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

impl<'a> State<'a> {
    pub fn next<P>(self, programs: &mut P, event: Event) -> State<'a>
    where
        P: Iterator<Item = &'a Program>,
    {
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
                        info!("Outta time - completing program: {:?}", &program);
                        let end_time = current_time + program.cooling_time;
                        State::Cooling { program, end_time }
                    } else {
                        State::Heating { program, end_time }
                    }
                }
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

    pub fn start<P>(programs: &mut P) -> State<'a>
    where
        P: Iterator<Item = &'a Program>,
    {
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

fn run_programs<'a, P, E>(programs: &mut P, events: E, duration: Duration) -> State<'a>
    where
        P: Iterator<Item = &'a Program>,
        E: IntoIterator<Item = Event>,
{
    let duration = duration.to_std().unwrap();
    let mut state = State::start(programs);
    for event in events {
        info!("{} <- {:?}", &state, &event);
        if state == State::Done { break; }
        state = state.next(programs, event);
        sleep(duration);
    }
    state
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

    run_programs(&mut config.programs(), events, Duration::seconds(1));
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use crate::{Event, run_programs, State};
    use crate::config::{HeaterPosition, Program};

    #[test]
    fn test_programs() {
        env_logger::init();
        let programs: Vec<Program> = vec![
            Program {
                heating_time: Duration::milliseconds(5),
                temp_sensor: String::from("TH1"),
                temp_abort: 80.0,
                thermostat: None,
                cooling_time: Duration::milliseconds(10),
                heater_position: HeaterPosition::Top,
                heater_duty: 1.0,
            },
            Program {
                heating_time: Duration::milliseconds(3),
                temp_sensor: String::from("J7"),
                temp_abort: 100.0,
                thermostat: Some(80.0),
                cooling_time: Duration::milliseconds(10),
                heater_position: HeaterPosition::Bottom,
                heater_duty: 1.0,
            },
        ];

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
        ];

        let final_state = run_programs(
            &mut programs.iter(),
            events,
            Duration::milliseconds(1),
        );
        assert_eq!(State::Done, final_state);
    }
}