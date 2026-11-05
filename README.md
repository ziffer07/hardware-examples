## Rust on hardware

In this repo we will be exploring arduino hardware code in rust.

### Installation so far

1. You need to install rustup and rustc
``` snap install --classic rustup ```
``` rustup toolchain install nightly ```

2. Check the ([datasheet](https://docs.arduino.cc/resources/datasheets/A000066-datasheet.pdf)) for Arduino UNO. It uses ATMega328P processor

3. ATMega328P's ([datasheet](https://ww1.microchip.com/downloads/en/DeviceDoc/en590320.pdf)) says it uses 8-bit AVR Risc based chip.

4. Check ([Rust platform support](https://doc.rust-lang.org/beta/rustc/platform-support.html)) and find avr-none target. Because it is tier 3 target we cannot simply install using rustup. Therefore, we will do the following.

