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
* `--no-vsync` - Disable vsync
* `--bootrom [path to bootrom file]` - Specify the path for a bootrom

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
- [The GameBoy Programming Manual](http://index-of.es/Varios-2/Game%20Boy%20Programming%20Manual.pdf)
- [gbdev gameboy sound hardware](https://gbdev.gg8.se/wiki/articles/Gameboy_sound_hardware)
- [Hactix's awsome blog post](https://hacktix.github.io/GBEDG/)
- [Nightshade's awsome blog post](https://nightshade256.github.io/2021/03/27/gb-sound-emulation.html)
- [The Ultimate GameBoy Talk](https://www.youtube.com/watch?v=HyzD8pNlpwI)