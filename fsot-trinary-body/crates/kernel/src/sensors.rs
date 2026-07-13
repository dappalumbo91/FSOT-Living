//! Hardware-backed interoception sensors (x86_64).
//!
//! - Heap: real linked_list_allocator used/free
//! - TSC: rdtsc delta
//! - Thermal: IA32_THERM_STATUS when available; QEMU-safe fallback

use linked_list_allocator::LockedHeap;

/// Used / capacity from the global heap allocator.
pub fn heap_usage(allocator: &LockedHeap, capacity: usize) -> (usize, usize) {
    let used = allocator.lock().used();
    (used.min(capacity), capacity)
}

#[inline]
pub fn rdtsc() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
            options(nomem, nostack, preserves_flags)
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

#[inline]
unsafe fn rdmsr(msr: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") lo,
        out("edx") hi,
        options(nomem, nostack, preserves_flags),
    );
    ((hi as u64) << 32) | (lo as u64)
}

fn cpuid_thermal_supported() -> bool {
    let eax: u32;
    let _ebx: u32;
    let _ecx: u32;
    let _edx: u32;
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {ebx_save:e}, ebx",
            "pop rbx",
            inout("eax") 6u32 => eax,
            ebx_save = out(reg) _ebx,
            out("ecx") _ecx,
            out("edx") _edx,
            options(nostack, preserves_flags),
        );
    }
    eax & 1 != 0
}

const IA32_THERM_STATUS: u32 = 0x19C;
const MSR_TEMPERATURE_TARGET: u32 = 0x1A2;

/// Thermal valence for interoception: **1.0 = cool/comfortable**, **0.0 = hot**.
/// Returns ~0.72 neutral on QEMU without host CPU passthrough.
pub fn cpu_temp_norm() -> f64 {
    if !cpuid_thermal_supported() {
        return 0.72;
    }
    let therm = unsafe { rdmsr(IA32_THERM_STATUS) };
    if therm & (1u64 << 31) == 0 {
        return 0.72;
    }
    let offset = ((therm >> 16) & 0x7F) as f64;
    let tj_raw = unsafe { rdmsr(MSR_TEMPERATURE_TARGET) };
    let mut tj = ((tj_raw >> 16) & 0xFF) as f64;
    if tj < 60.0 {
        tj = 100.0;
    }
    let temp_c = tj - offset;
    // Map 30°C → 1.0 cool, tj → 0.0 hot
    let cool = 30.0_f64;
    let span = (tj - cool).max(1.0);
    let hotness = ((temp_c - cool) / span).clamp(0.0, 1.0);
    (1.0 - hotness).clamp(0.05, 1.0)
}
