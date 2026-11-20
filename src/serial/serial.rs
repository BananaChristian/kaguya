use core::fmt::{self, Write};

use spin::Mutex;
use x86_64::instructions::{interrupts::without_interrupts, port::Port};

const COM1 : u16 = 0x3F8;
const COM1_LINE_STATUS: u16 = 0x3FD;

pub struct SerialPort;

impl SerialPort {
    fn write_byte(&mut self, byte: u8){
        unsafe{
            let mut status = Port::<u8>::new(COM1_LINE_STATUS);
            while status.read() & 0x20 == 0 {}
            let mut data = Port::<u8>::new(COM1);
            data.write(byte);
        }

    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes(){
            if byte == b'\n'{
                self.write_byte(b'\r');
            }
            self.write_byte(byte);
        }
        Ok(())
    }
}

static SERIAL: Mutex<SerialPort> = Mutex::new(SerialPort);

#[doc(hidden)]
pub fn _print(args: fmt::Arguments){
    without_interrupts(||{
        SERIAL.lock().write_fmt(args).unwrap()
    })
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
