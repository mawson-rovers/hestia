use std::{fs, io, thread, time};
use std::io::Write;

#[derive(Debug)]
pub(crate) struct Gpio {
    sysfp: fs::File,
}

impl Gpio {
    pub fn set_low(&mut self) -> io::Result<()> {
        self.sysfp.write_all(b"0")
    }

    pub fn set_high(&mut self) -> io::Result<()> {
        self.sysfp.write_all(b"1")
    }
}

pub(crate) fn open(gpio_num: u16) -> io::Result<Gpio> {
    export_gpio_if_unexported(gpio_num)?;

    // give time for the udev rules to create the gpio files
    // ref: https://stackoverflow.com/questions/39524234/bug-with-writing-to-file-in-linux-sys-class-gpio
    thread::sleep(time::Duration::from_millis(100));
    disable_active_low(gpio_num)?;

    thread::sleep(time::Duration::from_millis(100));
    set_gpio_output(gpio_num)?;

    thread::sleep(time::Duration::from_millis(100));
    let path = format!("/sys/class/gpio/gpio{}/value", gpio_num);
    Ok(Gpio {
        sysfp: fs::File::create(path)?
    })
}

fn export_gpio_if_unexported(gpio_num: u16) -> io::Result<()> {
    let file = format!("/sys/class/gpio/gpio{}", gpio_num);
    if let Err(_) = fs::metadata(&file) {
        let mut export_fp = fs::File::create("/sys/class/gpio/export")?;
        write!(export_fp, "{}", gpio_num)
    } else {
        Ok(())
    }
}

fn disable_active_low(gpio_num: u16) -> io::Result<()> {
    // ensure we're using '0' as low
    let mut low_file = fs::File::create(format!("/sys/class/gpio/gpio{}/active_low", gpio_num))?;
    low_file.write_all(b"0")
}

fn set_gpio_output(gpio_num: u16) -> io::Result<()> {
    let mut file = fs::File::create(format!("/sys/class/gpio/gpio{}/direction", gpio_num))?;
    file.write_all(b"out")
}
