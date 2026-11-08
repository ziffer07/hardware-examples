#![no_std]
#![no_main]

use core::ptr::read_volatile;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// Memory addresses for ATmega328P Port B registers
const DDRB: *mut u8 = 0x24 as *mut u8;
const PORTB: *mut u8 = 0x25 as *mut u8;
const PIN5: u8 = 1 << 5;

// Fixes Error 2: Wraps the unsafe attribute correctly for modern Nightly Rust
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    unsafe {
        *DDRB |= PIN5;
    }

    loop {
        unsafe {
            *PORTB |= PIN5;
            delay(500_000);

            *PORTB &= !PIN5;
            delay(500_000);
        }
    }
}



// Run this to stop the code from running
// #[unsafe(no_mangle)]
// pub extern "C" fn main() -> ! {
//     // No pin configurations, no register alterations
//     loop {
//         // The CPU will sit here forever doing nothing, keeping the LED off
//     }
// }


fn delay(count: u32) {
    let mut i = 0;
    while i < count {
        unsafe { read_volatile(DDRB as *const u8 ); }
        i += 1;
    }
}