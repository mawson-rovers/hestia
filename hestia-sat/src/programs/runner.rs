use std::fmt::Formatter;
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::thread::sleep;

use chrono::{DateTime, Duration, Utc};
use lazy_static::lazy_static;
use log::{debug, info};

use crate::board::{Board, BoardId};
use crate::heater::{HeaterMode, TargetSensor};
use crate::payload::Payload;

use crate::programs::{Program, Programs};

#[derive(Debug)]
pub enum State<'a> {
    Heating {
        program: &'a Program,
        end_time: DateTime<Utc>,
    },
    Cooling {
        program: &'a Program,
    },
    FinishedProgram,
    Done,
    Failed {
        message: String,
    },
}

impl<'a> PartialEq for State<'a> {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other),
            (State::Heating { .. }, State::Heating { .. }) |
            (State::Cooling { .. }, State::Cooling { .. }) |
            (State::Done, State::Done) |
            (State::Failed { .. }, State::Failed { .. }))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event<'a> {
    TemperatureReading {
        board: BoardId,
        temp_sensor: &'a str,
        temp: f32,
    },
    Time,
}

impl<'a> State<'a> {
    /// Returns a new State if one is entered, otherwise None indicates current state continues
    pub fn next(&self, controller: &mut PayloadController<'a>, event: Event) -> Option<State<'a>> {
        let current_time = Utc::now();
        match self {
            &State::Heating { program, end_time } => {
                if current_time >= end_time {
                    info!("Heating time completed: {}", program);
                    return Some(controller.start_cool(program));
                }
                match event {
                    Event::TemperatureReading { board, temp_sensor, temp }
                    if board == program.heat_board && temp_sensor == program.temp_sensor => {
                        debug!("Checking {}, {}, temp {}°C vs abort temp: {}°C",
                            board, temp_sensor, temp, program.temp_abort);
                        if temp > program.temp_abort {
                            info!("Abort temp reached ({} > {}): {}", temp, program.temp_abort, program);
                            Some(controller.start_cool(program))
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
            &State::Cooling { program } => {
                match event {
                    Event::TemperatureReading { board, temp_sensor, temp }
                    if board == program.heat_board && temp_sensor == program.temp_sensor => {
                        debug!("Checking {}, {}, temp {}°C vs cool temp: {}°C",
                            board, temp_sensor, temp, program.cool_temp);
                        if temp <= program.cool_temp {
                            info!("Cool temp reached ({} <= {}): {}", temp, program.cool_temp, program);
                            Some(State::FinishedProgram)
                        } else {
                            None
                        }
                    }
                    _ => None
                }
            },
            &State::FinishedProgram => {
                Some(controller.next_program_or_done())
            },
            &State::Done => None,
            state => Some(State::Failed {
                message: format!("Invalid event {:#?} for state: {:#?}", event, state)
            })
        }
    }
}

impl std::fmt::Display for State<'_> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            State::Heating { program, end_time } => {
                write!(f, "State::Heating(end_time: {}, temp_abort: {:.2}, thermostat: {})",
                       end_time.format("%T.%3f"), program.temp_abort,
                       program.thermostat.map(|v| format!("{:.2}", v))
                           .unwrap_or(String::from("#empty")))
            }
            State::Cooling { program } => write!(f, "State::Cooling({})", program.cool_temp),
            State::FinishedProgram => write!(f, "State::FinishedProgram"),
            State::Done => write!(f, "State::Done"),
            State::Failed { message } => write!(f, "State::Failed(\"{}\")", message),
        }
    }
}

lazy_static! {
    static ref ABORT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

pub struct PayloadController<'a> {
    payload: &'a Payload,
    programs: &'a mut dyn Iterator<Item=&'a Program>,
}

impl<'a> PayloadController<'a> {
    pub fn new(payload: &'a Payload, programs: &'a mut dyn Iterator<Item=&'a Program>) -> Self {
        // switch off heater if program is stopped with Ctrl-C
        ctrlc::set_handler(|| {
            ABORT.store(true, Relaxed);
        }).expect("Error setting Ctrl-C handler");

        PayloadController { payload, programs }
    }

    pub fn run(&mut self, events: &mut dyn Iterator<Item = Event<'a>>, duration: Duration) -> State<'a>
    {
        let duration = duration.to_std().unwrap();
        let mut state = self.start();
        for event in events {
            debug!("{} <- {:?}", &state, &event);
            if state == State::Done { break }
            if let Some(new_state) = state.next(self, event) {
                state = new_state
            }
            sleep(duration);
            if self.is_aborted() { break }
        }
        state
    }

    pub fn start(&mut self) -> State<'a> {
        let first = self.programs.next().expect("Didn't find any programs");
        self.start_heat(first)
    }

    pub fn start_heat(&self, program: &'a Program) -> State<'a> {
        info!("Starting heat for program: {:?}", &program);
        let board = &self.payload[program.heat_board as u8];
        let end_time = Utc::now() + program.heat_time;
        board.write_heater_duty((program.heat_duty * 255.0) as u16);
        let target_sensor = TargetSensor::from(program.temp_sensor.clone());
        board.write_target_sensor(target_sensor);
        match program.thermostat {
            Some(temp) => {
                board.write_target_temp(temp);
                board.write_heater_mode(HeaterMode::PID);
            }
            None => {
                board.write_heater_mode(HeaterMode::PWM);
            }
        }
        State::Heating { program, end_time }
    }

    pub fn start_cool(&self, program: &'a Program) -> State<'a> {
        info!("Starting cool for program: {:?}", &program);
        let board = &self.payload[program.heat_board as u8];
        board.write_heater_mode(HeaterMode::OFF);
        State::Cooling { program }
    }

    pub fn next_program_or_done(&mut self) -> State<'a> {
        if let Some(program) = self.programs.next() {
            self.start_heat(program)
        } else {
            State::Done
        }
    }

    pub fn is_aborted(&self) -> bool {
        ABORT.load(Relaxed)
    }
}

impl<'a> Drop for PayloadController<'a> {
    fn drop(&mut self) {
        info!("Disabling payload heaters");
        for board in self.payload {
            board.write_heater_mode(HeaterMode::OFF);
        }
    }
}


pub struct PayloadEvents<'a> {
    payload: &'a Payload,
    buffer: Vec<Event<'a>>,
    phantom: PhantomData<&'a Event<'a>>,
}

impl<'a> PayloadEvents<'a> {
    pub fn new(payload: &'a Payload) -> PayloadEvents<'a> {
        PayloadEvents { payload, buffer: vec![], phantom: PhantomData }
    }
}

impl<'a> Iterator for PayloadEvents<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            self.buffer.push(Event::Time); // always put something in the buffer
            for board in self.payload {
                if let Some(reading) = read_board(board, board.into()) {
                    self.buffer.push(reading);
                }
            }
        }
        self.buffer.pop()
    }
}

fn read_board<'a>(board: &Board, heat_board: BoardId) -> Option<Event<'a>> {
    let sensor = board.get_target_sensor().ok()?;
    let reading = board.read_target_sensor_temp().ok()?;
    Some(Event::TemperatureReading {
        board: heat_board,
        temp_sensor: sensor.id,
        temp: reading.display_value,
    })
}

pub fn run(payload: &Payload, programs: &Programs) {
    loop {
        let mut events = PayloadEvents::new(payload);
        let program_list = &mut programs.iter();
        let mut controller = PayloadController::new(payload, program_list);
        controller.run(&mut events, Duration::seconds(1));
        if !programs.run_loop || controller.is_aborted() { break; }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use crate::board::BoardId;
    use crate::payload::Payload;

    use crate::programs::Program;
    use crate::programs::runner::{Event, PayloadController, PayloadEvents, State};

    const TH1: &str = "TH1";
    const J7: &str = "J7";

    #[test]
    fn test_programs() {
        let _ = env_logger::try_init();
        let programs: Vec<Program> = vec![
            Program {
                id: 0,
                name: String::from("Top"),
                heat_time: Duration::milliseconds(5),
                temp_sensor: String::from("TH1"),
                temp_abort: 80.0,
                thermostat: None,
                cool_temp: 40.0,
                heat_board: BoardId::Top,
                heat_duty: 1.0,
            },
            Program {
                id: 1,
                name: String::from("Bottom"),
                heat_time: Duration::milliseconds(3),
                temp_sensor: String::from("J7"),
                temp_abort: 100.0,
                thermostat: Some(80.0),
                cool_temp: 30.0,
                heat_board: BoardId::Bottom,
                heat_duty: 1.0,
            },
        ];

        let events: Vec<Event> = vec![
            Event::Time,
            Event::TemperatureReading { board: BoardId::Top, temp: 55.0, temp_sensor: TH1 },
            Event::TemperatureReading { board: BoardId::Top, temp: 105.0, temp_sensor: TH1 },
            Event::Time,
            Event::Time,
            Event::TemperatureReading { board: BoardId::Top, temp: 60.0, temp_sensor: TH1 },
            Event::TemperatureReading { board: BoardId::Bottom, temp: 80.0, temp_sensor: J7 },
            Event::Time,
            Event::TemperatureReading { board: BoardId::Top, temp: 35.0, temp_sensor: TH1 },
            Event::TemperatureReading { board: BoardId::Bottom, temp: 120.0, temp_sensor: J7 },
            Event::TemperatureReading { board: BoardId::Bottom, temp: 100.0, temp_sensor: J7 },
            Event::Time,
            Event::Time,
            Event::TemperatureReading { board: BoardId::Top, temp: 120.0, temp_sensor: TH1 },
            Event::TemperatureReading { board: BoardId::Bottom, temp: 90.0, temp_sensor: J7 },
            Event::Time,
            Event::Time,
            Event::TemperatureReading { board: BoardId::Top, temp: 60.0, temp_sensor: TH1 },
            Event::TemperatureReading { board: BoardId::Bottom, temp: 60.0, temp_sensor: J7 },
            Event::Time,
            Event::Time,
            Event::Time,
            Event::TemperatureReading { board: BoardId::Bottom, temp: 30.0, temp_sensor: J7 },
            Event::Time,
        ];

        let payload = Payload::create();
        let program_list = &mut programs.iter();
        let mut controller = PayloadController::new(&payload, program_list);
        let final_state = controller.run(
            &mut events.into_iter(),
            Duration::milliseconds(1),
        );
        assert_eq!(State::Done, final_state);
    }

    #[test]
    fn test_events_from_single_board() {
        let _ = env_logger::try_init();
        let payload = Payload::single_board(1);
        let mut events = PayloadEvents::new(&payload);
        let board = BoardId::Top;
        assert_eq!(Some(Event::TemperatureReading { board, temp_sensor: "TH1", temp: 25.191437 }),
                   events.next());
        assert_eq!(Some(Event::Time),
                   events.next());
        assert_eq!(Some(Event::TemperatureReading { board, temp_sensor: "TH1", temp: 25.191437 }),
                   events.next());
        assert_eq!(Some(Event::Time),
                   events.next());

        let payload = Payload::single_board(2);
        let mut events = PayloadEvents::new(&payload);
        let board = BoardId::Bottom;
        assert_eq!(Some(Event::TemperatureReading { board, temp_sensor: "TH1", temp: 25.191437 }),
                   events.next());
        assert_eq!(Some(Event::Time),
                   events.next());
        assert_eq!(Some(Event::TemperatureReading { board, temp_sensor: "TH1", temp: 25.191437 }),
                   events.next());
        assert_eq!(Some(Event::Time),
                   events.next());
    }
}