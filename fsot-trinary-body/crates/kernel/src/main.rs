//! FSOT Trinary Body — bare-metal soma.
//!
//! Zygote → heap → sensors → interoception → genetic fabric → organ membrane → Mind ABI.

#![no_std]
#![no_main]

extern crate alloc;

mod sensors;
mod serial;

use body::{SensorInputs, Soma};
use bootloader_api::{config::Mapping, entry_point, BootInfo, BootloaderConfig};
use core::panic::PanicInfo;
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

static mut HEAP_MEM: [u8; 512 * 1024] = [0; 512 * 1024];
const HEAP_CAP: usize = 512 * 1024;

const CONFIG: BootloaderConfig = {
    let mut cfg = BootloaderConfig::new_default();
    cfg.mappings.physical_memory = Some(Mapping::Dynamic);
    cfg
};

entry_point!(kernel_main, config = &CONFIG);

fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    unsafe {
        let heap_start = core::ptr::addr_of_mut!(HEAP_MEM) as *mut u8;
        ALLOCATOR.lock().init(heap_start, HEAP_CAP);
    }

    serial::init();
    serial::println("[fsot-body] zygote online (trinary genetic soma)");
    serial::println("[fsot-body] sensors: heap(real) tsc thermal_msr|proxy");
    serial::println("[fsot-body] layers: interoception | Mind ABI | organ membrane");
    serial::println("[fsot-body] MOTOR STATUS|ATTEND|BREATHE|REPORT|EXPRESS|REPRESS|ORGANS|READ|WRITE|NOTE");
    serial::println("[fsot-body] type MOTOR lines on this serial (QEMU -serial stdio)");

    let mut soma = Soma::boot();
    serial::println("[fsot-body] soma boot complete");
    for line in soma.drain_outbox() {
        serial::println(&line);
    }

    soma.handle_line("MOTOR WRITE /organs/bridge kernel_online trinary_body");
    for line in soma.drain_outbox() {
        serial::println(&line);
    }

    let mut last_tsc = sensors::rdtsc();

    loop {
        while let Some(b) = serial::try_read() {
            soma.handle_byte(b);
        }

        let tsc_now = sensors::rdtsc();
        let tsc_delta = tsc_now.saturating_sub(last_tsc) as f64;
        last_tsc = tsc_now;

        let (heap_used, heap_cap) = sensors::heap_usage(&ALLOCATOR, HEAP_CAP);
        let cpu_temp = sensors::cpu_temp_norm();

        let sensors_in = SensorInputs {
            cpu_temp_norm: cpu_temp,
            tsc_delta,
            heap_used,
            heap_cap,
            uptime_seconds: 0.0,
            breath_phase: 0.0,
        };

        let _sample = soma.tick(sensors_in);

        for line in soma.drain_outbox() {
            serial::println(&line);
        }

        for _ in 0..150_000 {
            core::hint::spin_loop();
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial::println("[fsot-body] PANIC");
    if let Some(loc) = info.location() {
        serial::print("  at ");
        serial::print(loc.file());
        serial::print(":");
        serial::print_u64(loc.line() as u64);
        serial::println("");
    }
    loop {
        core::hint::spin_loop();
    }
}
