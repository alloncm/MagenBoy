# MagenBoy

A GameBoy emulator developed by me.

The main goal of this project is to be able to play Pokemon on my own emulator.

## Implemented Cartridges Types
- Rom (No MBC controller)
- MBC1
- MBC3

**More will be added if neccessary (and by neccessary I mean if games I want to play will require them)**

## How to use

### Building

```shell
cargo build --release --features [optional_features]
```
#### Optional features:
* `sdl` - Link to sdl2 (On by default)
* `static-sdl` - will link statically to sdl2 
On by default 
* `sdl-resample` - Use the audio resampler from sdl2 library and a manual one I wrote
* `push-audio` - Use a push methododlogy instead of pull for the delivery of the sound samples to sdl2
* `static-scale` - Will use a fixed scale values for the renderer instead of addapting to the screen size
* `u16pixel` - pixels are represented by 16 bits and not 32 bits - neccessary for interfacing the ili9341 spi lcd
* `apu` - Turn on the apu (On by default)
* `rpi` - Input is from the RPI GPIO pins and output is to an ili9341 spi lcd connected to the RPI GPIO pins, activates the `u16pixel` feature.
* `mmio` - Will interface the spi lcd screen using the Memory Mapped IO interface of the RPI for better performance (uses the DMA peripherals as well, activates the `rpi` feature.
* `terminal-menu` - replace the gui menu with a terminal menu, since it is more capable this is the defualt

> **Note** to turn off on by default features pass `--no-default-features` when building

### Key bindings:

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

### Running

#### Desktop
```sh
magenboy [path_to_rom] [other_optional_flags]
```

#### Raspberry Pi
See - [RealMagenBoy](docs/RealMagenBoy.md)

### Optional flags

* `--log` - Print logs in debug mode to a file
* `--file-audio` - Saves the audio to a file
* `--full-screen` - Full screen mode
* `--no-vsync` - Disable vsync
* `--bootrom [path to bootrom file]` - Specify the path for a bootrom (If not specified the emualtor will look for `dmg_boot.bin` at the cwd)
* `--rom-menu [path to roms folder]` - Opens an interactive dialog uopn start to choose the rom from the folder
Choose a game with the Joypad bindings (Dpad and A to confirm)

## GameBoy

### Development Status

- CPU - Cycle accurate CPU
- PPU - Cycle accurate fifo PPU
- Timer - Mostly accurate timer
- APU - mostly accurate APU
- Tests
    - [Blargg's cpu_instrs](https://github.com/retrio/gb-test-roms/tree/master/cpu_instrs) - :thumbsup:
    - [dmg-acid2](https://github.com/mattcurrie/dmg-acid2) - :thumbsup:
    - [TurtleTests](https://github.com/Powerlated/TurtleTests) - :thumbsup:
    - [CPU cycle accurate](https://github.com/retrio/gb-test-roms/tree/master/instr_timing) - :thumbsup:
    - [mooneye-test-suite](https://github.com/Gekkio/mooneye-test-suite)
        - acceptance/ppu/intr_2_0_timing - :thumbsup:
        - acceptance/ppu/intr_2_mode0_timing - :thumbsup:
        - acceptance/ppu/intr_2_mode3_timing - :thumbsup:
        - acceptance/ppu/intr_2_oam_ok_timing - :thumbsup:
    - APU passes some of [blargs dmg_sound tests](https://github.com/retrio/gb-test-roms/tree/master/dmg_sound)- :thumbsup:
    - Timer passes most of [mooneye-test-suite](https://github.com/Gekkio/mooneye-test-suite/tree/main/acceptance/timer) - :thumbsup:

### Games Tested
- Pokemon Red - :thumbsup:
- Tetris - :thumbsup:
- Super Mario World - :thumbsup:

## GameBoy Color

Curerently there is no Support (support is planned in the future)

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