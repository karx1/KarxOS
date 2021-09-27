// ATA Driver!
use crate::println;
use alloc::{string::String, vec::Vec};
use bit_field::BitField;
use core::hint::spin_loop;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::{Port, PortReadOnly, PortWriteOnly};

// Commands to send to the drives
#[repr(u16)]
enum Command {
    Read = 0x20,
    Write = 0x30,
    Identify = 0xEC,
}

#[allow(dead_code)]
#[repr(usize)]
enum Status {
    ERR = 0,
    IDX,
    CORR,
    DRQ,
    SRV,
    DF,
    RDY,
    BSY,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Bus {
    id: u8,
    irq: u8,

    data_register: Port<u16>,
    error_register: PortReadOnly<u8>,
    features_register: PortWriteOnly<u8>,
    sector_count_register: Port<u8>,
    lba0_register: Port<u8>,
    lba1_register: Port<u8>,
    lba2_register: Port<u8>,
    drive_register: Port<u8>,
    status_register: PortReadOnly<u8>,
    command_register: PortWriteOnly<u8>,

    alternate_status_register: PortReadOnly<u8>,
    control_register: PortWriteOnly<u8>,
    drive_blockless_register: PortReadOnly<u8>,
}

#[allow(dead_code)]
impl Bus {
    pub fn new(id: u8, io_base: u16, ctrl_base: u16, irq: u8) -> Self {
        Bus {
            id,
            irq,

            data_register: Port::new(io_base),
            error_register: PortReadOnly::new(io_base + 1),
            features_register: PortWriteOnly::new(io_base + 1),
            sector_count_register: Port::new(io_base + 2),
            lba0_register: Port::new(io_base + 3),
            lba1_register: Port::new(io_base + 4),
            lba2_register: Port::new(io_base + 5),
            drive_register: Port::new(io_base + 6),
            status_register: PortReadOnly::new(io_base + 7),
            command_register: PortWriteOnly::new(io_base + 7),

            alternate_status_register: PortReadOnly::new(ctrl_base),
            control_register: PortWriteOnly::new(ctrl_base),
            drive_blockless_register: PortReadOnly::new(ctrl_base + 1),
        }
    }

    fn reset(&mut self) {
        use crate::clock::nanowait;
        unsafe {
            self.control_register.write(4);
            nanowait(5);
            self.control_register.write(0);
            nanowait(2000);
        }
    }

    fn wait(&mut self) {
        for _ in 0..4 {
            unsafe {
                self.alternate_status_register.read();
            }
        }
    }
    fn select_drive(&mut self, drive: u8) {
        let drive_id = 0xA0 | (drive << 4);
        unsafe {
            self.drive_register.write(drive_id);
        }
    }

    fn write_command(&mut self, cmd: Command) {
        unsafe {
            self.command_register.write(cmd as u8);
        }
    }

    fn status(&mut self) -> u8 {
        unsafe { self.status_register.read() }
    }

    fn lba1(&mut self) -> u8 {
        unsafe { self.lba1_register.read() }
    }

    fn lba2(&mut self) -> u8 {
        unsafe { self.lba2_register.read() }
    }

    fn read_data(&mut self) -> u16 {
        unsafe { self.data_register.read() }
    }

    fn busy_loop(&mut self) {
        self.wait();
        let start = crate::clock::uptime();
        while self.is_busy() {
            if crate::clock::uptime() - start > 1.0 {
                return self.reset();
            }

            spin_loop();
        }
    }

    fn is_busy(&mut self) -> bool {
        self.status().get_bit(Status::BSY as usize)
    }

    fn is_error(&mut self) -> bool {
        self.status().get_bit(Status::ERR as usize)
    }

    fn is_ready(&mut self) -> bool {
        self.status().get_bit(Status::RDY as usize)
    }

    pub fn identify_drive(&mut self, drive: u8) -> Option<[u16; 256]> {
        self.reset();
        self.wait();
        self.select_drive(drive);
        unsafe {
            self.sector_count_register.write(0);
            self.lba0_register.write(0);
            self.lba1_register.write(0);
            self.lba2_register.write(0);
        }

        self.write_command(Command::Identify);

        if self.status() == 0 {
            println!("status 0");
            return None;
        }

        self.busy_loop();

        if self.lba1() != 0 || self.lba2() != 0 {
            println!("lba thingies");
            return None;
        }

        for i in 0.. {
            if i == 256 {
                println!("i 256");
                self.reset();
                return None;
            }
            if self.is_error() {
                println!("Is error");
                return None;
            }
            if self.is_ready() {
                println!("ready");
                break;
            }
        }

        let mut res = [0; 256];
        for i in 0..256 {
            res[i] = self.read_data();
        }

        Some(res)
    }
}

lazy_static! {
    pub static ref BUS: Mutex<Bus> = Mutex::new(Bus::new(0, 0x1F0, 0x3F6, 14));
}

fn disk_size(sectors: u32) -> (u32, String) {
    let bytes = sectors * 512;
    if bytes >> 20 < 1000 {
        (bytes >> 20, String::from("MB"))
    } else {
        (bytes >> 30, String::from("GB"))
    }
}

pub fn info() -> Vec<(u8, String, String, u32, String)> {
    use x86_64::registers::control::{Cr0Flags, Cr0};
    let mut flags = Cr0::read();
    flags.set(Cr0Flags::WRITE_PROTECT, false);
    unsafe {Cr0::write(flags)};
    let mut bus = BUS.lock();
    let mut res = Vec::new();
    for drive in 0..2 {
        if let Some(buf) = bus.identify_drive(drive) {
            let mut serial = String::new();
            for i in 10..20 {
                for &b in &buf[i].to_be_bytes() {
                    serial.push(b as char);
                }
            }
            serial = serial.trim().into();
            let mut model = String::new();
            for i in 27..47 {
                for &b in &buf[i].to_be_bytes() {
                    model.push(b as char);
                }
            }
            model = model.trim().into();
            let sectors = (buf[61] as u32) << 16 | (buf[60] as u32);
            let (size, unit) = disk_size(sectors);
            res.push((drive, model, serial, size, unit));
        } else {
            println!("No drive found!");
        }
    }
    res
}
