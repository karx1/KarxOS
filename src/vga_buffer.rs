use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::port::Port;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub(crate) struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ascii_character: u8,
    pub(crate) color_code: ColorCode,
}

// This is characters, not pixels
pub(crate) const BUFFER_HEIGHT: usize = 25;
pub(crate) const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    pub chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT], // Use Volatile for futureproofing reads/writes
}

pub struct Writer {
    pub column_position: usize,
    color_code: ColorCode,
    pub buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position < BUFFER_WIDTH {
                    let row = BUFFER_HEIGHT - 1;
                    let col = self.column_position;

                    let color_code = self.color_code;
                    self.buffer.chars[row][col].write(ScreenChar {
                        ascii_character: byte,
                        color_code,
                    });
                    self.column_position += 1;
                }
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Only write printable ASCII characters
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        // Move all the rows up
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }

        // Clear top row
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) }
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // Turn off interrupts to avoid a deadlock
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn backspace() {
    let mut writer = WRITER.lock();
    let row = BUFFER_HEIGHT - 1;
    let col = writer.column_position;
    let color_code = writer.color_code;
    // Barrier for prompt
    if col != 4 {
        writer.buffer.chars[row][col - 1].write(ScreenChar {
            ascii_character: b' ',
            color_code,
        });
        writer.column_position -= 1;
    }
}

pub struct Cursor {
    port_low: Port<u8>,
    port_high: Port<u8>,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            port_low: Port::new(0x3D4),
            port_high: Port::new(0x3D5),
        }
    }

    fn move_cursor(&mut self, pos: u16) {
        unsafe {
            self.port_low.write(0x0F);
            self.port_high.write((pos & 0xFF) as u8);
            self.port_low.write(0x0E);
            self.port_high.write(((pos >> 8) & 0xFF) as u8);
        }
    }
}

lazy_static! {
    pub static ref CURSOR: Mutex<Cursor> = Mutex::new(Cursor::new());
}

pub fn move_cursor(x: u16, y: u16) {
    let pos: u16 = y * (BUFFER_WIDTH as u16) + x;
    let mut cursor = CURSOR.lock();
    cursor.move_cursor(pos);
}

pub fn change_color(foreground: Color, background: Color) {
    let mut writer = WRITER.lock();
    let color = ColorCode::new(foreground, background);
    writer.color_code = color;
}
