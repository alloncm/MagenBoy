# MagenBoy

A GameBoy emulator developed by me.

The main goal of this project is to be able to play Pokemon on my own emulator.

## Implemented Cartridges Types
- Rom (No MBC controller)
- MBC1
- MBC3

**More will be added if neccessary (and by neccessary I mean if games I want to play will require them)**

## GameBoy

### Development Status

- [Blargg's cpu_instrs](https://github.com/retrio/gb-test-roms/tree/master/cpu_instrs) - :thumbsup:
- [dmg-acid2](https://github.com/mattcurrie/dmg-acid2) - :thumbsup:
- [TurtleTests](https://github.com/Powerlated/TurtleTests) - :thumbsup:
- Accurate emulation - 
    - [CPU cycle accurate](https://github.com/retrio/gb-test-roms/tree/master/instr_timing) - :thumbsup:
    - PPU currently opcoce accurate - :thumbsup:
    - APU currently cycle accurate, passes some of [blargs dmg_sound tests](https://github.com/retrio/gb-test-roms/tree/master/dmg_sound)- :thumbsup:
    - Timer cycle acurate, passes most of [mooneye-gb tests](https://github.com/wilbertpol/mooneye-gb/tree/master/tests/acceptance/timer) - :thumbsup:

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