use gpio::GpioOut;
use log::info;

#[cfg(target_os = "linux")]
pub fn gpio_set_low(pin: u16) {
    if let Ok(mut pin) = gpio::sysfs::SysFsGpioOutput::open(pin) {
        let _ = pin.set_low();
    }
}

#[cfg(target_os = "linux")]
pub fn gpio_set_high(pin: u16) {
    if let Ok(mut pin) = gpio::sysfs::SysFsGpioOutput::open(pin) {
        let _ = pin.set_high();
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