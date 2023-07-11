pub mod i2c;
pub mod msp430;
pub mod ads7828;
pub mod max31725;

#[cfg(not(target_os = "linux"))]
pub mod stub_i2c;