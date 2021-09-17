use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::println;
use crate::print;
use lazy_static::lazy_static;
use crate::gdt;
use pic8259::ChainedPics;
use spin::Mutex;


lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION : DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;
    use pc_keyboard::{layouts, DecodedKey, KeyCode, HandleControl, Keyboard, ScancodeSet1};

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = {
            Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore))
        };
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    if character == '\u{8}' {
                        crate::vga_buffer::backspace();
                    } else if character == '\u{9}' {
                        print!("    ");
                    } else if character == '\n' {
                        use arrayvec::ArrayString;
                        let writer = crate::vga_buffer::WRITER.lock();
                        
                        let mut builder = ArrayString::<80>::new();
                        for character in &writer.buffer.chars[crate::vga_buffer::BUFFER_HEIGHT - 1] {
                            builder.push(character.read().ascii_character as char);
                        }
                        
                        unsafe {
                            crate::vga_buffer::WRITER.force_unlock();
                        }
                        crate::shell::evaluate(&builder);
                    } else {
                        print!("{}", character)
                    }

                    let writer = crate::vga_buffer::WRITER.lock();
                    let col = writer.column_position;
                    let row = crate::vga_buffer::BUFFER_HEIGHT - 1;

                    crate::vga_buffer::move_cursor(col as u16, row as u16);
                },
                DecodedKey::RawKey(key) => {
                    match key {
                        // TODO
                        KeyCode::ArrowLeft => {
                            let mut writer = crate::vga_buffer::WRITER.lock();
                            let col = writer.column_position;
                            let row = crate::vga_buffer::BUFFER_HEIGHT - 1;
                            
                            if col != 4 {
                                crate::vga_buffer::move_cursor((col as u16) - 1, row as u16);
                                writer.column_position -= 1;
                            }
                        },
                        KeyCode::ArrowRight => {
                            let mut writer = crate::vga_buffer::WRITER.lock();
                            let col = writer.column_position;
                            let row = crate::vga_buffer::BUFFER_HEIGHT - 1;

                            crate::vga_buffer::move_cursor((col as u16) + 1, row as u16);
                            writer.column_position += 1;
                        },
                        _ => {}                        
                    }
                }, 
            }
        }
    }

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}
