use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use x86_64::instructions::port::Port;

static CLOCKS_PER_NANOSECOND: AtomicU64 = AtomicU64::new(0);
static PIT_TICKS: AtomicUsize = AtomicUsize::new(0);
const PIT_FREQUENCY: f64 = 3_579_545.0 / 3.0;
const PIT_DIVIDER: usize = 1193;
const PIT_INTERVAL: f64 = (PIT_DIVIDER as f64) / PIT_FREQUENCY;

fn rdtsc() -> u64 {
    unsafe {
        core::arch::x86_64::_mm_lfence();
        core::arch::x86_64::_rdtsc()
    }
}

fn set_pit_freqency_divider(divider: u16, channel: u8) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        let bytes = divider.to_le_bytes();
        let mut cmd: Port<u8> = Port::new(0x43);
        let mut data: Port<u8> = Port::new(0x40 + channel as u16);
        let operating_mode = 6;
        let access_mode = 3;
        unsafe {
            cmd.write((channel << 6) | (access_mode << 4) | operating_mode);
            data.write(bytes[0]);
            data.write(bytes[1]);
        }
    })
}

fn ticks() -> usize {
    PIT_TICKS.load(Ordering::Relaxed)
}

pub fn uptime() -> f64 {
    PIT_INTERVAL * ticks() as f64
}

pub fn sleep(seconds: f64) {
    let start = uptime();
    while uptime() - start < seconds {
        x86_64::instructions::interrupts::enable_and_hlt();
    }
}

pub fn pit_interrupt_handler() {
    PIT_TICKS.fetch_add(1, Ordering::Relaxed);
}

pub fn init() {
    let divider = if PIT_DIVIDER < 65535 { PIT_DIVIDER } else { 0 };
    let channel = 0;
    set_pit_freqency_divider(divider as u16, channel);

    let calibration_time = 250_000;
    let a = rdtsc();
    sleep(calibration_time as f64 / 1e6);
    let b = rdtsc();
    CLOCKS_PER_NANOSECOND.store((b - a) / calibration_time, Ordering::Relaxed);
}
