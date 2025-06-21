# NX (Nintendo Switch)

## Setup your Switch

I have used this [guide](https://switch.hacks.guide/user_guide/getting_started.html) to setup my Switch to be able to run homebrew apps

## Build

Verify `docker` or `podman` are installed as the build process use the devkitpro image to get reliable access to the tools.

Run:
```sh
cargo make nx
```

## Install 

Copy the `magenboy.nro` to the switch or upload it with nxlink (needs to be installed with the devkitpro toolchain)

nxlink command after pressing `Y` in the homebrew app:

```sh
nxlink -s path_to_magenboy.nro
```

if it fails verify the PC can ping the switch IP and if it can add `-a ip_address` to the command flags.

## Run

The app expect to find ROMS in a folder called `roms` in the path if the app.

### Usage

| Joypad     | Joycon      |
| ---------- | ----------- |
| A          | A or X      |
| B          | B or Y      |
| Start      | +           |
| Select     | -           |
| Dpad Up    | Up arrow    |
| Dpad Down  | Down arrow  |
| Dpad Left  | Left arrow  |
| Dpad Right | Right arrow |

The menu can be opened by pressing `L` + `R`