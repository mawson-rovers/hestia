mod programs;

use std::fmt::Formatter;
use std::marker::PhantomData;
use std::thread::sleep;
use chrono::{DateTime, Duration, Utc};
use log::info;
use uts_ws1::heater::{HeaterMode, TargetSensor};
use uts_ws1::payload::{Config, Payload};
use crate::programs::{HeatBoard, Program, Programs};

#[derive(Debug)]
enum State<'a> {
    Heating {
        payload: &'a Payload,
        program: &'a Program,
        end_time: DateTime<Utc>,
    },
    Cooling {
        payload: &'a Payload,
        end_time: DateTime<Utc>,
    },
    Done,
    Failed {
        message: String,
    },
}

impl<'a> PartialEq for State<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (State::Heating { .. }, State::Heating { .. }) => true,
            (State::Cooling { .. }, State::Cooling { .. }) => true,
            (State::Done, State::Done) => true,
            (State::Failed { .. }, State::Failed { .. }) => true,
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[derive(Debug)]
enum Event<'a> {
    TemperatureReading {
        board: HeatBoard,
        temp_sensor: &'a str,
        temp: f32,
    },
    Time,
}

impl<'a> State<'a> {
    /// Returns a new State if one is entered, otherwise None indicates current state continues
    pub fn next<P>(&self, programs: &mut P, event: Event) -> Option<State<'a>>
        where
            P: Iterator<Item=&'a Program>,
    {
        let current_time = Utc::now();
        match self {
            &State::Heating { payload, program, end_time } => match event {
                Event::TemperatureReading { board, temp_sensor, temp }
                if board == program.heat_board && temp_sensor == program.temp_sensor
                => {
                    info!("Checking {}, {}, temp {}°C vs abort temp: {}°C",
                        board, temp_sensor, temp, program.temp_abort);
                    if temp > program.temp_abort {
                        info!("Too hot - aborting program: {:?}", &program);
                        Some(Self::start_cool(payload, program))
                    } else {
                        None
                    }
                }
                Event::Time => {
                    if current_time >= end_time {
                        info!("Outta time - completing program: {:?}", &program);
                        Some(Self::start_cool(payload, program))
                    } else {
                        None
                    }
                }
                _ => None,
            },
            &State::Cooling { payload, end_time } => match event {
                Event::Time if current_time >= end_time => {
                    if let Some(program) = programs.next() {
                        Some(Self::start_heat(payload, program))
                    } else {
                        Some(State::Done)
                    }
                }
                _ => {
                    None
                }
            },
            &State::Done => None,
            state => Some(State::Failed {
                message: format!("Invalid event {:#?} for state: {:#?}", event, state)
            })
        }
    }

    pub fn start<P>(payload: &'a Payload, programs: &mut P) -> State<'a>
        where
            P: Iterator<Item=&'a Program>,
    {
        let first = programs.next().expect("Didn't find any programs");
        State::start_heat(payload, first)
    }

    pub fn start_heat(payload: &'a Payload, program: &'a Program) -> State<'a> {
        info!("Starting heat for program: {:?}", &program);
        let board = &payload[program.heat_board as u8];
        let end_time = Utc::now() + program.heat_time;
        board.write_heater_duty((program.heat_duty * 255.0) as u16);
        let target_sensor = TargetSensor::from(program.temp_sensor.clone());
        board.write_target_sensor(target_sensor);
        match program.thermostat {
            Some(temp) => {
                board.write_target_temp(temp);
                board.write_heater_mode(HeaterMode::PID);
            },
            None => {
                board.write_heater_mode(HeaterMode::PWM);
            }
        }
        State::Heating { payload, program, end_time }
    }

    pub fn start_cool(payload: &'a Payload, program: &'a Program) -> State<'a> {
        info!("Starting cool for program: {:?}", &program);
        let board = &payload[program.heat_board as u8];
        board.write_heater_mode(HeaterMode::OFF);
        let end_time = Utc::now() + program.cool_time;
        State::Cooling { payload, end_time }
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

fn run_programs<'a, P, E>(payload: &'a Payload, programs: &mut P, events: E, duration: Duration)
    -> State<'a>
    where
        P: Iterator<Item=&'a Program>,
        E: IntoIterator<Item=Event<'a>>,
{
    let duration = duration.to_std().unwrap();
    let mut state = State::start(payload, programs);
    for event in events {
        info!("{} <- {:?}", &state, &event);
        if state == State::Done { break; }
        if let Some(new_state) = state.next(programs, event) {
            state = new_state
        }
        sleep(duration);
    }
    state
}

struct PayloadEvents<'a> {
    payload: &'a Payload,
    last_board: HeatBoard,
    phantom: PhantomData<&'a Event<'a>>,
}

impl<'a> PayloadEvents<'a> {
    fn new(payload: &'a Payload) -> PayloadEvents<'a> {
        PayloadEvents { payload, last_board: HeatBoard::Bottom, phantom: PhantomData }
    }
}

impl<'a> Iterator for PayloadEvents<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let board = match self.last_board {
            HeatBoard::Top => HeatBoard::Bottom,
            HeatBoard::Bottom => HeatBoard::Top,
        };
        self.last_board = board;
        match self.payload[board as u8].get_target_sensor() {
            Ok(sensor) => {
                match self.payload[board as u8].read_target_sensor_temp() {
                    Ok(reading) => {
                        Some(Event::TemperatureReading {
                            board: self.last_board,
                            temp_sensor: sensor.label,
                            temp: reading.display_value,
                        })
                    }
                    Err(_) => Some(Event::Time),
                }
            }
            Err(_) => Some(Event::Time),
        }
    }
}


pub fn main() {
    let config = Config::read();
    let programs = Programs::load(&config);
    info!("Loaded programs:\n{:#?}", programs);

    let payload = Payload::from_config(&config);
    let events = PayloadEvents::new(&payload);
    run_programs(&payload, &mut programs.iter(), events, Duration::seconds(1));
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use uts_ws1::payload::Payload;
    use crate::{Event, run_programs, State};
    use crate::programs::{HeatBoard, Program};

    const TH1: &str = "TH1";
    const J7: &str = "J7";

    #[test]
    fn test_programs() {
        env_logger::init();
        let programs: Vec<Program> = vec![
            Program {
                heat_time: Duration::milliseconds(5),
                temp_sensor: String::from("TH1"),
                temp_abort: 80.0,
                thermostat: None,
                cool_time: Duration::milliseconds(10),
                heat_board: HeatBoard::Top,
                heat_duty: 1.0,
            },
            Program {
                heat_time: Duration::milliseconds(3),
                temp_sensor: String::from("J7"),
                temp_abort: 100.0,
                thermostat: Some(80.0),
                cool_time: Duration::milliseconds(10),
                heat_board: HeatBoard::Bottom,
                heat_duty: 1.0,
            },
        ];

        let events: Vec<Event> = vec![
            Event::Time,
            Event::TemperatureReading { board: HeatBoard::Top, temp: 55.0, temp_sensor: TH1 },
            Event::TemperatureReading { board: HeatBoard::Top, temp: 105.0, temp_sensor: TH1 },
            Event::Time,
            Event::Time,
            Event::TemperatureReading { board: HeatBoard::Top, temp: 60.0, temp_sensor: TH1 },
            Event::TemperatureReading { board: HeatBoard::Bottom, temp: 80.0, temp_sensor: J7 },
            Event::Time,
            Event::Time,
            Event::TemperatureReading { board: HeatBoard::Bottom, temp: 120.0, temp_sensor: J7 },
            Event::TemperatureReading { board: HeatBoard::Bottom, temp: 100.0, temp_sensor: J7 },
            Event::Time,
            Event::Time,
            Event::TemperatureReading { board: HeatBoard::Top, temp: 120.0, temp_sensor: TH1 },
            Event::TemperatureReading { board: HeatBoard::Bottom, temp: 90.0, temp_sensor: J7 },
            Event::Time,
            Event::Time,
            Event::TemperatureReading { board: HeatBoard::Top, temp: 60.0, temp_sensor: TH1 },
            Event::TemperatureReading { board: HeatBoard::Bottom, temp: 60.0, temp_sensor: J7 },
            Event::Time,
            Event::Time,
            Event::Time,
            Event::Time,
            Event::Time,
        ];

        let payload = Payload::create();
        let final_state = run_programs(
            &payload,
            &mut programs.iter(),
            events,
            Duration::milliseconds(1),
        );
        assert_eq!(State::Done, final_state);
    }
}