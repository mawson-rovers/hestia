use std::env;

use log::info;

use crate::{heater, I2cBus, ReadResult};
use crate::sensors::{Sensor, SensorInterface};

static ALL_SENSORS: &[Sensor] = &[
    Sensor::new("TH1", SensorInterface::MSP430, 0x01, "Centre", -42.0135, 43.18),
    Sensor::new("TH2", SensorInterface::MSP430, 0x02, "Top-left of heater", -35.7124, 54.61),
    Sensor::new("TH3", SensorInterface::MSP430, 0x03, "Bottom-right of heater", -53.88, 33.496),
    Sensor::new("U4", SensorInterface::MAX31725, 0x48, "Top-left", -15.976, 75.225),
    Sensor::new("U5", SensorInterface::MAX31725, 0x4F, "Top-right", 81.788, 75.692),
    Sensor::new("U6", SensorInterface::MAX31725, 0x49, "Bottom-right", -82.296, 12.8535),
    Sensor::new("U7", SensorInterface::MAX31725, 0x4B, "Centre", 46.228, 47.752),
    Sensor::new("TH4", SensorInterface::ADS7828, 0x00, "Centre", -45.8705, 43.18),
    Sensor::new("TH5", SensorInterface::ADS7828, 0x01, "Top-right", -77.9814, 75.0769),
    Sensor::new("TH6", SensorInterface::ADS7828, 0x02, "Bottom-left of heater", 33.274, 30.226),
    Sensor::mounted("J7", SensorInterface::MSP430, 0x04),
    Sensor::mounted("J8", SensorInterface::MSP430, 0x05),
    Sensor::mounted("J9", SensorInterface::MSP430, 0x06),
    Sensor::mounted("J10", SensorInterface::MSP430, 0x07),
    Sensor::mounted("J11", SensorInterface::MSP430, 0x08),
    Sensor::mounted("J12", SensorInterface::ADS7828, 0x03),
    Sensor::mounted("J13", SensorInterface::ADS7828, 0x04),
    Sensor::mounted("J14", SensorInterface::ADS7828, 0x05),
    Sensor::mounted("J15", SensorInterface::ADS7828, 0x06),
    Sensor::mounted("J16", SensorInterface::ADS7828, 0x07),
];

const SENSOR_DISABLE_ENV_VAR: &str = "UTS_SENSOR_DISABLE";

pub struct Board {
    pub bus: I2cBus,
    pub sensors: Vec<Sensor>,
    pub center_sensor: Sensor,
    pub thermostat_sensor: Sensor,
}

impl Board {
    pub fn init(bus: I2cBus) -> Board {
        let mut sensors = Vec::from(ALL_SENSORS);
        if env::var(SENSOR_DISABLE_ENV_VAR).is_ok() {
            let disabled_var = env::var(SENSOR_DISABLE_ENV_VAR)
                .unwrap_or("".to_string())
                .to_uppercase();
            let disabled: Vec<&str> = disabled_var
                .split(",")
                .collect();
            info!("Disabling sensors per configuration: {:?}", disabled);
            sensors.retain(|s| !disabled.contains(&s.id.to_uppercase().as_str()))
        }
        Board {
            bus,
            sensors: sensors.clone(),
            center_sensor: sensors[0],
            thermostat_sensor: sensors[0],
        }
    }

    pub fn read_center_temp(&self) -> ReadResult<f32> {
        self.center_sensor.read_temp(&self.bus)
    }

    pub fn read_temps(&self) -> Vec<ReadResult<f32>> {
        self.sensors.iter().map(|s| s.read_temp(&self.bus)).collect()
    }

    pub fn is_heater_enabled(&self) -> bool {
        return heater::is_enabled(&self.bus);
    }
}

