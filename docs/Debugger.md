# MagenBoy Debugger

The debugger functionality is baked into the core project and can be enabled with the `dbg` compilation feature.

Currently only the SDL frontend supports it and offers a command line based UI for most of the commands.

## SDL frontend terminal commands

command | shortcut | description |  example
--------| -------- | ----------- | ----------
halt    | h        | Halts the program execution in order to interact with the debugger | `halt`
continue | c       | Continue the program execution (could be stopped again by entering the halt or by break points or watch points the user registered) | `continue`
step | s           | Step the program 1 instruction    | `step`
skip_halt | - | skip untill CPU is hanlted
break [address:bank] | b [address:bank]         | Set a breakpoint at the given address, will break right before the instruction at this address is about to be executed | `break 0x1234:1`
remove_break [address:bank] | rb [address:bank] | Remove a break point by the address | `remove_break 0x1234:0`
registers | reg | Display the registers values | `registers`
disassemble [number_of_opcodes] | di [number_of_opcodes] | Display a disassembly of the current program counter | `disassemble 10`
dump [address number_of_bytes] | du [address number_of_bytes] | Display a memory dump of the current bank at specific address | `dump 0x40 10`
watch [address:bank] | w [address:bank] | Set a watch point at the given address | `watch 0xFFFF:0`
remove_watch [address:bank] | rw [address:bank] | Remove a watch point by the address | `remove_watch 0xFFFF:0`
ppu_info | pi | Display info about the current state of the pixel processing unit | `ppu_info`
ppu_layer [layer] | pl [layer] | Render all the tiles in a given layer of the PPU memory, possible layers - [bg (background), win (window), spr (sprites/objects)] | `ppu_layer bg`