#[cfg(target_os = "linux")]
use gpio::GpioOut;

#[cfg(target_os = "linux")]
use log::error;

use log::info;

/// Equivalent of uts_en.sh on WS-1
// echo 0 > /sys/class/gpio/gpio45/value
// echo 1 > /sys/class/gpio/gpio47/value
// echo 0 > /sys/class/gpio/gpio27/value
pub fn enable_payload() {
    gpio_set_low(45);
    gpio_set_high(47);
    gpio_set_low(27);
}

pub fn disable_payload() {
    gpio_set_low(45);
    gpio_set_low(47);
    gpio_set_low(27);
}

#[cfg(target_os = "linux")]
pub fn gpio_set_low(pin: u16) {
    if let Ok(mut gpio) = gpio::sysfs::SysFsGpioOutput::open(pin) {
        info!("Setting GPIO pin {} to low", pin);
        let _ = gpio.set_low();
    } else {
        error!("Could not write to GPIO pin: {}", pin);
    }
}

#[cfg(target_os = "linux")]
pub fn gpio_set_high(pin: u16) {
    if let Ok(mut gpio) = gpio::sysfs::SysFsGpioOutput::open(pin) {
        info!("Setting GPIO pin {} to high", pin);
        let _ = gpio.set_high();
    } else {
        error!("Could not write to GPIO pin: {}", pin);
    }
}

#[cfg(not(target_os = "linux"))]
pub fn gpio_set_low(_pin: u16) {
    info!("GPIO operations only implemented for Linux");
}

#[cfg(not(target_os = "linux"))]
pub fn gpio_set_high(_pin: u16) {
    info!("GPIO operations only implemented for Linux");
}