# MagenBoy

A GameBoy emulator developed by me.

The main goal of this project is to be able to play Pokemon on my own emulator.

## Implemented Cartridges Types
- Rom (No MBC controller)
- MBC1
- MBC3
- MBC5

**More will be added if neccessary (and by neccessary I mean if games I want to play will require them)**

## Building

Install `cargo-make`
```sh
cargo install cargo-make
```
verify you have docker or podman installed 

### Desktop

```sh
cargo make sdl
```

or with more configuration options:

```shell
cargo build --release --package gb --features [optional_features]
```
#### Optional features:
* `sdl` - Link to sdl2 (On by default)
* `static-sdl` - will link statically to sdl2 
On by default 
* `sdl-resample` - Use the audio resampler from sdl2 library and a manual one I wrote
* `push-audio` - Use a push methododlogy instead of pull for the delivery of the sound samples to sdl2
* `u16pixel` - pixels are represented by 16 bits and not 32 bits - neccessary for interfacing the ili9341 spi lcd
* `apu` - Turn on the apu (On by default)
* `rpi` - Input is from the RPI GPIO pins and output is to an ili9341 spi lcd connected to the RPI GPIO pins, activates the `u16pixel` feature.
* `mmio` - Will interface the spi lcd screen using the Memory Mapped IO interface of the RPI for better performance (uses the DMA peripherals as well, activates the `rpi` feature.

> **Note** to turn off on by default features pass `--no-default-features` when building

#### Key bindings:

| Joypad     | Keyboard    |
| ---------- | ----------- |
| A          | X           |
| B          | Z           |
| Start      | S           |
| Select     | A           |
| Dpad Up    | Up arrow    |
| Dpad Down  | Down arrow  |
| Dpad Left  | Left arrow  |
| Dpad Right | Right arrow |

### Raspberry Pi Baremetal (with ili9341 display and gpio buttons)

```sh
cargo make rpi_baremetal
```

or manually:

1. Install the rust nightly toolchain for `armv7a-none-eabihf`:
```shell
rustup target add armv7a-none-eabihf --toolchain nightly
rustup +nightly component add rust-src
```

Unfurtuantly `armv7a-none-eabihf` is a [tier3](https://doc.rust-lang.org/nightly/rustc/platform-support.html#tier-3) target for the Rust compiler so building for it requires the nightly toolchain - [source](https://stackoverflow.com/questions/67352828/how-to-build-for-tier-3-target-not-included-in-rustup-target-list)

> **Notice** Verify that you install the `armv7a-none-eabihf` target and not the `armv7a-none-eabi` target, as the later doesn't have support for hardware floats.

2. Install Cargo Binutils:
```shell
cargo install cargo-binutils
rustup component add llvm-tools-preview
```

3. Edit the relevant settings in `configuration.rs`
4. Run `build.sh` or `build.bat` which will compile and create an bootable image called `kernel7.img`

## Running

### Desktop
```sh
magenboy [path_to_rom] [other_optional_flags]
```

### Raspberry Pi Desktop with peripherals
See - [RealMagenBoy](docs/RealMagenBoy.md)

#### Optional flags

* `--file-audio` - Saves the audio to a file
* `--full-screen` - Full screen mode
* `--no-vsync` - Disable vsync
* `--bootrom [path to bootrom file]` - Specify the path for a bootrom (If not specified the emualtor will look for `dmg_boot.bin` at the cwd)
* `--rom-menu [path to roms folder]` - Opens an interactive dialog uopn start to choose the rom from the folder
Choose a game with the Joypad bindings (Dpad and A to confirm)
* `--mode [mahcine mode]` - Override the auto machine detection for the game (mode can be: `CGB` - Gameboy color | `DMG` - Original Gameboy  | `ANY` - default)
* `--shutdown-rpi` - Requires `rpi` feature, shutdown the RPi upon shutdown of the program

### Raspberry Pi Baremetal

Currently only Raspberry Pi 4 is supported using the following instructions:
* Format a sd card and make a single `FAT32` partition called `boot`
* Copy the file `config.txt` to the root dir of the sd card
* Copy the following files from the [Raspberry Pi firmware repo](https://github.com/raspberrypi/firmware/tree/master/boot) onto the SD card:
    - [fixup4.dat](https://github.com/raspberrypi/firmware/raw/master/boot/fixup4.dat)
    - [start4.elf](https://github.com/raspberrypi/firmware/raw/master/boot/start4.elf)
    - [bcm2711-rpi-4-b.dtb](https://github.com/raspberrypi/firmware/raw/master/boot/bcm2711-rpi-4-b.dtb)
* Copy `kernel7.img` onto the SD card
* Connect all the peripherals (ili9341 display and gpio buttons)
* Insert the SD card to the RPI4 and boot it

_**Note**: Should it not work on your RPi4, try renaming `start4.elf` to `start.elf` (without the 4)
on the SD card._

### QEMU

Currently Qemu doesn't support RPI4 in 32 bit mode, so in order to test it I added support for RPI2 mapping.
To change to RPI2 mode build with the `rpi2` feature and not the default `rpi4` feature.

running with qemu:
```shell
qemu-system-arm.exe -M raspi2b -serial null -serial mon:stdio -kernel path_to_elf
```

_**Note** Qemu takes the path to the elf generated by cargo not the image generated by binutils_
the UART output will be written to the console.

I think that not all the peripherals I use are implemented in QEMU so I used this mainly to debug boot and CPU initialization problems

## GameBoy

### Development Status

- CPU - Cycle accurate CPU
- PPU - Cycle accurate fifo PPU
- Timer - Mostly accurate timer
- APU - mostly accurate APU
- Tests
    - [Blargg's cpu_instrs](https://github.com/retrio/gb-test-roms/tree/master/cpu_instrs)
    - [dmg-acid2](https://github.com/mattcurrie/dmg-acid2) 
    - [TurtleTests](https://github.com/Powerlated/TurtleTests)
    - [CPU cycle accurate](https://github.com/retrio/gb-test-roms/tree/master/instr_timing)
    - [mooneye-test-suite](https://github.com/Gekkio/mooneye-test-suite)
        - acceptance/ppu/intr_2_0_timing
        - acceptance/ppu/intr_2_mode0_timing 
        - acceptance/ppu/intr_2_mode3_timing 
        - acceptance/ppu/intr_2_oam_ok_timing 
    - APU passes some of [blargs dmg_sound tests](https://github.com/retrio/gb-test-roms/tree/master/dmg_sound)
    - Timer passes most of [mooneye-test-suite](https://github.com/Gekkio/mooneye-test-suite/tree/main/acceptance/timer)

### Games Tested

- Pokemon Red 
- Tetris 
- Super Mario World 
- The Legend of Zelda: A Link to the Past 

## GameBoy Color

### Developement Status

- CPU - Full support
- PPU - Kinds of works
- Tests
    - [cgb-acid2](https://github.com/mattcurrie/cgb-acid2) 
    - [MagenTests](https://github.com/alloncm/MagenTests) 

### Games Tested

- Pokemon Yellow
- Tetris DX

## Resources
### Gameboy
- [The Pandocs](https://gbdev.io/pandocs/)
- [gbops](https://izik1.github.io/gbops/index.html)
- [The GameBoy Programming Manual](https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=&ved=2ahUKEwi2muaT98j4AhWwhc4BHRaxAaEQFnoECAcQAQ&url=https%3A%2F%2Farchive.org%2Fdownload%2FGameBoyProgManVer1.1%2FGameBoyProgManVer1.1.pdf&usg=AOvVaw3LoEvXhZRBH7r68qdXIhiP)
- [gbdev gameboy sound hardware](https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware)
- [Hactix's awsome blog post](https://hacktix.github.io/GBEDG/)
- [Nightshade's awsome blog post](https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html)
- [The Ultimate GameBoy Talk](https://www.youtube.com/watch?v=HyzD8pNlpwI)
- [Nitty gritty Gameboy timing](http://blog.kevtris.org/blogfiles/Nitty%20Gritty%20Gameboy%20VRAM%20Timing.xt)
- [mgba gbdoc](https://mgba-emu.github.io/gbdoc/)

### RaspberryPi
- [Raspberry Pi docs](https://www.raspberrypi.com/documentation/computers/processors.html)
- [juj/fbcp-ili9341 as a refference](https://github.com/juj/fbcp-ili9341)
- [Raspberry Pi DMA programming in C](https://iosoft.blog/2020/05/25/raspberry-pi-dma-programming/)
- [Ili9341 docs](https://cdn-shop.adafruit.com/datasheets/ILI9341.pdf)
#### BareMetal RaspberryPi
- [Bare-metal Boot Code for ARMv8-A](http://classweb.ece.umd.edu/enee447.S2021/baremetal_boot_code_for_ARMv8_A_processors.pdf)
- [Low performance Baremetal code Blog post](https://forums.raspberrypi.com/viewtopic.php?t=219212)
- [Raspberry-Pi-Multicode examples by LdB-ECM](https://github.com/LdB-ECM/Raspberry-Pi)
- [RaspberryPi official Linux fork](https://github.com/raspberrypi/linux)
- ARM official Docs
    - [ARM Cortex-A72 manual](https://developer.arm.com/documentation/100095/0003)
    - [ARMv7-A Architecture Reference Manual](https://developer.arm.com/documentation/ddi0406/cb/?lang=en)
    - [ARMv8-A Architecture Reference Manual](https://developer.arm.com/documentation/ddi0487/ia/?lang=en)
    - [ARMv8-A Registers](https://developer.arm.com/documentation/ddi0595/2021-12/AArch32-Registers/CCSIDR--Current-Cache-Size-ID-Register?lang=en)
    - [ARMv7-A programmer Guide](https://developer.arm.com/documentation/den0013/latest/)