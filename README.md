# MagenBoy

A GameBoy emulator developped by me.

The main goal of this project is to be able to play Pokemon on my own emulator.

## Implemented Cartridges Types
- Rom (No MBC controller)
- MBC1
- MBC3

#### More will be added if neccessary (and by neccessary I mean if games I want to play will require them)

## Development Status

### GameBoy
- Blargg's cpu_instrs (https://github.com/retrio/gb-test-roms/tree/master/cpu_instrs) - :thumbsup:
- dmg-acid2 (https://github.com/mattcurrie/dmg-acid2) - :thumbsup:
- TurtleTests (https://github.com/Powerlated/TurtleTests) - :thumbsup:
- Accurate emulation - 
    - CPU cycle accurate (https://github.com/retrio/gb-test-roms/tree/master/instr_timing) - :thumbsup:
    - PPU currently opcoce accurate - :thumbsup:
    - APU currently cycle accurate - :thumbsup:
    - Timer cycle acurate, passes most of mooneye-gb tests (https://github.com/wilbertpol/mooneye-gb/tree/master/tests/acceptance/timer) - :thumbsup:

#### Games Booting
- Pokemon Red - :thumbsup:
- Tetris - :thumbsup:

### GameBoyColor - :x:
- implement the new PPU
- Support new roms