#![no_std]
#![no_main]
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

const DDRB: *mut u8 = 0x24 as *mut u8;
const PORTB: *mut u8 = 0x25 as *mut u8;

const LED_PIN: u8 = 1 << 5; // Bitwise operator 32 in rust 00100000

// UART registers


// 1. USART control and status register A. RX is at PORTD 0, but we are operating in bits so we change directly Atmega328p's receiver values
// It's 7th bit check if RXCn reads if data is coming or not
const UCSR0A: *mut u8 = 0xC0 as *mut u8;

// 2. USART control and status register B. We enable bit 4 which enables the USART receiver. This is done when initializing. Also, value of bit 2 of 
// this register is used to determine the number of bits of data which is being transmitted. if 1 then its either reserved or 9 bits.
const UCSR0B: *mut u8 = 0xC1 as *mut u8;

// 3. USART control and status register C. This is used to specify how many bits we are transmitting, is we are using parity bit of not and how many stop bits (1 or 2)
const UCSR0C: *mut u8 = 0xC2 as *mut u8;

// L and H basically mean high and low byte here. It stores baud rate at a particulat freequency. For our 16 MHz, our value is 103 which can be stored in low byte as (01100111).
const UBRR0L: *mut u8 = 0xC4 as *mut u8;
// H will mostly be 0 unless we have value over 255 which is more than what an u8 can store.
const UBRR0H: *mut u8 = 0xC5 as *mut u8;

// Reading this part means reading the data bits that we receive. This is the real data.
const UDR0: *mut u8 = 0xC6 as *mut u8;


// Functions needed for the program to run

// Init the baud rate (9600 for this example)
// Check if byte has arrived (Basically see if 7th bit & 1<<7 has 1 or != 0)
// Read the value from u8. 

#[unsafe(no_mangle)]
pub extern "C" fn main() {
    unsafe {
        write_volatile(DDRB, LED_PIN);
    }
    loop{}
}
