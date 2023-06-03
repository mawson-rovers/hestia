use crate::ReadResult;

#[repr(u16)]
#[derive(Debug, Copy, Clone)]
pub enum HeaterMode {
    OFF = 0x00,
    /// temperature controlled
    PID = 0x01,
    /// fixed power input
    PWM = 0x02,
}

pub trait Heater {
    fn read_mode(&self) -> ReadResult<HeaterMode>;
    fn read_mode_raw(&self) -> ReadResult<u16>;
    fn write_mode(&self, mode: HeaterMode);

    fn read_duty(&self) -> ReadResult<u8>;
    fn read_duty_raw(&self) -> ReadResult<u16>;
    fn write_duty(&self, duty: u8);

    fn read_target_temp(&self) -> ReadResult<f32>;
    fn read_target_temp_raw(&self) -> ReadResult<u16>;
    fn write_target_temp(&self, temp: f32);

    fn read_target_sensor(&self) -> ReadResult<u16>;
    fn write_target_sensor(&self, target_sensor: u8);
}

impl std::fmt::Display for HeaterMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaterMode::OFF => write!(f, "OFF"),
            HeaterMode::PID => write!(f, "PID"),
            HeaterMode::PWM => write!(f, "PWM"),
        }
    }
}

impl std::convert::TryFrom<u16> for HeaterMode {
    type Error = ();

    fn try_from(v: u16) -> Result<Self, Self::Error> {
        match v {
            x if x == HeaterMode::OFF as u16 => Ok(HeaterMode::OFF),
            x if x == HeaterMode::PID as u16 => Ok(HeaterMode::PID),
            x if x == HeaterMode::PWM as u16 => Ok(HeaterMode::PWM),
            _ => Err(()),
        }
    }
}
