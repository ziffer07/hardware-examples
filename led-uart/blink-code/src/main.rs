#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// GPIO registers
const DDRB: *mut u8 = 0x24 as *mut u8;
const PORTB: *mut u8 = 0x25 as *mut u8;

// USART registers
const UCSR0A: *mut u8 = 0xC0 as *mut u8;
const UCSR0B: *mut u8 = 0xC1 as *mut u8;
const UCSR0C: *mut u8 = 0xC2 as *mut u8;

const UBRR0L: *mut u8 = 0xC4 as *mut u8;
const UBRR0H: *mut u8 = 0xC5 as *mut u8;

const UDR0: *mut u8 = 0xC6 as *mut u8;

const LED_PIN: u8 = 1 << 5;

fn usart_initialize() {
    unsafe {
        // 9600 baud at 16 MHz
        write_volatile(UBRR0H, 0);
        write_volatile(UBRR0L, 103);

        // Enable USART receiver and transmitter
        write_volatile(UCSR0B, (1 << 4) | (1 << 3));

        // Asynchronous mode, 8 data bits, no parity, 1 stop bit
        write_volatile(UCSR0C, 0x06);
    }
}

fn usart_available() -> bool {
    unsafe {
        read_volatile(UCSR0A) & (1 << 7) != 0
    }
}

fn usart_read() -> u8 {
    unsafe {
        read_volatile(UDR0)
    }
}

fn usart_write(byte: u8) {
    unsafe {
        while read_volatile(UCSR0A) & (1 << 5) == 0 {}
        write_volatile(UDR0, byte);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    unsafe {
        // Configure PB5 as output
        write_volatile(DDRB, read_volatile(DDRB) | LED_PIN);
    }

    usart_initialize();

    loop {
        if usart_available() {
            let command = usart_read();

            if command == b'?' {
                usart_write(b'R');
            } else {
                unsafe {
                    if command == b'1' {
                        write_volatile(
                            PORTB,
                            read_volatile(PORTB) | LED_PIN,
                        );
                        usart_write(command);
                    } else if command == b'0' {
                        write_volatile(
                            PORTB,
                            read_volatile(PORTB) & !LED_PIN,
                        );
                        usart_write(command);
                    }
                }
            }
        }
    }
}