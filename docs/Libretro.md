# Libretro

Since libretro offers cross platform support out of the box (mainly using [RetroArch](https://github.com/libretro/RetroArch)) 
I imlemented a libretro core for MagenBoy mainly to play MagenBoy on android.

## How to build

The command is `cargo make libretro`, which you'll need to have `cargo-make` to run (as the readme tells).

This command will build both the native desktop target and the `aarch64-linux-android` target and output a `.info` file 
along with a dynamic library at the same directory.

In order to build android you'll also need to install the android SDK and enable the NDK package.

Make sure to have an environment variable named `ANDROID_NDK_HOME` and set to the NDK install path, 
for example - `export ANDROID_NDK_HOME=/home/alloncm/Android/Sdk/ndk`.

## How to install

I'll explain how to install on RetroArch, if you are using another frontend - good luck!

First of course youll need to download RetroArch for your platform.

### Desktop

You can always run the rom from the command line using the generated dynamic library:

```sh
retroarch -v -L target/release/libmagenboy_libretro.so path_to_rom
```

But in order to install it as a core youll first need to suplly RetroArch with the metadata for the core (otherwise it will install in incorrectly).

1. Choose: `Settings -> Directory -> Core Info`, this is the path where RetroArch searches for the mathcing `.info` files for the cores, you need to copy the `.info` file to this directory.

2. Install the core, choose: `Load Core -> Install or Restore Core` and then navigate to the `.so` file and select it.

3. Verify installation by choosing: `Information -> Core Information` and check that the metadata is correct.

If there is no `Core Information` option there was an error in the installation process, manually delete the `.so` from the default directory configured in `Settings -> Directory -> Core` and start again.

Now you should be able to load roms and use the whole other options of RetroArch along with MagenBoy!

### Android

This instalation is a bit more complicated then the above since I assume you dont have root on the device (like I dont have) so will need to do some workarounds.

The installation process is kind of the same as the desktop except that without root user you can't execute the first step because you dont have access the internal directories the app uses by default.

In order to workaround this limitation we will change the default `.info` directory from an internal one to an external one in order to gain access

Choose: `Settings -> Directory -> Core Info` and set the path to another folder which you have a write access to (using a file explorer of your choice)
for example `/storage/emulated/0/RetroArch/info`.

Copy the `.info` to this folder, and continue as described in the desktop installation.

> Note: If you have root access just copy the `.info` to the configured directory.

> Warning: Do not attempt this workaround with the Core section (the `.so` files), since on android only certain directories can contain executable files.

## Input mapping

Each GameBoy button is mapped to the corresponding button in Libretro's joypad except for the A and B buttons,
A is mapped to both A and X and B is mapped to both B and Y.

I do that in order to make pressing them easier, especially on mobile.