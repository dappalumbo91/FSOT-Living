//! COM1 serial — first sensory/motor nerve of the body.
//! Minimal no_std UART for QEMU `-serial stdio`.

use core::fmt::Write;
use spin::Mutex;

const COM1: u16 = 0x3F8;

struct SerialPort;

static SERIAL: Mutex<SerialPort> = Mutex::new(SerialPort);

impl SerialPort {
    fn init(&self) {
        unsafe {
            outb(COM1 + 1, 0x00); // disable interrupts
            outb(COM1 + 3, 0x80); // DLAB on
            outb(COM1 + 0, 0x03); // divisor low (38400)
            outb(COM1 + 1, 0x00); // divisor high
            outb(COM1 + 3, 0x03); // 8N1
            outb(COM1 + 2, 0xC7); // FIFO
            outb(COM1 + 4, 0x0B); // IRQs, RTS/DSR
        }
    }

    fn write_byte(&self, b: u8) {
        unsafe {
            while inb(COM1 + 5) & 0x20 == 0 {
                core::hint::spin_loop();
            }
            outb(COM1, b);
        }
    }

    fn try_read_byte(&self) -> Option<u8> {
        unsafe {
            if inb(COM1 + 5) & 0x01 == 0 {
                None
            } else {
                Some(inb(COM1))
            }
        }
    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            if b == b'\n' {
                self.write_byte(b'\r');
            }
            self.write_byte(b);
        }
        Ok(())
    }
}

pub fn init() {
    SERIAL.lock().init();
}

pub fn print(s: &str) {
    let _ = SERIAL.lock().write_str(s);
}

pub fn println(s: &str) {
    let mut sp = SERIAL.lock();
    let _ = sp.write_str(s);
    let _ = sp.write_str("\n");
}

pub fn try_read() -> Option<u8> {
    SERIAL.lock().try_read_byte()
}

pub fn print_u64(mut n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 20;
    if n == 0 {
        print("0");
        return;
    }
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    print(core::str::from_utf8(&buf[i..]).unwrap_or("?"));
}

pub fn print_hex_u64(n: u64) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut buf = [0u8; 16];
    for i in 0..16 {
        let shift = (15 - i) * 4;
        buf[i] = HEX[((n >> shift) & 0xF) as usize];
    }
    print(core::str::from_utf8(&buf).unwrap_or("?"));
}

pub fn print_f64(v: f64) {
    if v < 0.0 {
        print("-");
        print_f64_abs(-v);
    } else {
        print_f64_abs(v);
    }
}

fn print_f64_abs(v: f64) {
    let int_part = v as u64;
    let frac = ((v - int_part as f64) * 100.0) as u64;
    print_u64(int_part);
    print(".");
    if frac < 10 {
        print("0");
    }
    print_u64(frac);
}

#[inline]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nomem, nostack, preserves_flags));
}

#[inline]
unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!("in al, dx", out("al") val, in("dx") port, options(nomem, nostack, preserves_flags));
    val
}
