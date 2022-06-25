# MagenBoy

A GameBoy emulator developed by me.

The main goal of this project is to be able to play Pokemon on my own emulator.

## Implemented Cartridges Types
- Rom (No MBC controller)
- MBC1
- MBC3

**More will be added if neccessary (and by neccessary I mean if games I want to play will require them)**

## How to use

```shell
magenboy [path_to_rom] [other_optional_flags]
```

### Optional flags

* `--log` - Print logs in debug mode to a file
* `--file-audio` - Saves the audio to a file
* `--full-screen` - Full screen mode
* `--no-vsync` - Disable vsync
* `--bootrom [path to bootrom file]` - Specify the path for a bootrom (If not specified the emualtor will look for `dmg_boot.bin` at the cwd)
* `--rom_menu [path to roms folder]` - Opens an interactive dialog uopn start to choose the rom from the folder

### Building

```shell
cargo build --release --features [optional_features]
```
#### Optional features:
* `static-sdl` - will link statically to sdl2.
On by default (to turn off pass `--no-default-features`)
* `sdl-resample` - Use the audio resampler from sdl2 library and a manual one I wrote
* `push-audio` - Use a push methododlogy instead of pull for the delivery of the sound samples to sdl2
* `static-scale` - Will use a fixed scale values for the renderer instead of addapting to the screen size
* `u16pixel` - pixels are represented by 16 bits and not 32 bits - neccessary for interfacing the ili9341 spi lcd
* `rpi` - Input is from the RPI GPIO pins and output is to an ili9341 spi lcd connected to the RPI GPIO pins, activates the `u16pixel` feature.
* `mmio` - Will interface the spi lcd screen using the Memory Mapped IO interface of the RPI for better performance (uses the DMA peripherals as well, activates the `rpi` feature.

## GameBoy

### Development Status

- CPU - Cycle accurate CPU
- PPU - Cycle accurate fifo PPU
- Timer - Mostly accurate timer
- APU - Cycle mostly accurate APU
- Tests
    - [Blargg's cpu_instrs](https://github.com/retrio/gb-test-roms/tree/master/cpu_instrs) - :thumbsup:
    - [dmg-acid2](https://github.com/mattcurrie/dmg-acid2) - :thumbsup:
    - [TurtleTests](https://github.com/Powerlated/TurtleTests) - :thumbsup:
    - [CPU cycle accurate](https://github.com/retrio/gb-test-roms/tree/master/instr_timing) - :thumbsup:
    - APU passes some of [blargs dmg_sound tests](https://github.com/retrio/gb-test-roms/tree/master/dmg_sound)- :thumbsup:
    - Timer passes most of [mooneye-gb tests](https://github.com/Gekkio/mooneye-gb/tree/master/tests/acceptance/timer) - :thumbsup:

### Games Tested
- Pokemon Red - :thumbsup:
- Tetris - :thumbsup:

## GameBoy Color

Curerently there is no Support (support is planned in the future)

## Resources
- [The Pandocs](https://gbdev.io/pandocs/)
- [gbops](https://izik1.github.io/gbops/index.html)
- [The GameBoy Programming Manual](https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=&ved=2ahUKEwi2muaT98j4AhWwhc4BHRaxAaEQFnoECAcQAQ&url=https%3A%2F%2Farchive.org%2Fdownload%2FGameBoyProgManVer1.1%2FGameBoyProgManVer1.1.pdf&usg=AOvVaw3LoEvXhZRBH7r68qdXIhiP)
- [gbdev gameboy sound hardware](https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware)
- [Hactix's awsome blog post](https://hacktix.github.io/GBEDG/)
- [Nightshade's awsome blog post](https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html)
- [The Ultimate GameBoy Talk](https://www.youtube.com/watch?v=HyzD8pNlpwI)
