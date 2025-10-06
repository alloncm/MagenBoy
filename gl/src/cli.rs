use argh::FromArgs;

#[derive(FromArgs)]
/// MagenBoy OpenGL frontend
pub struct CliArgs {
    /// path to the ROM file to load
    #[argh(option, short = 'r')]
    pub rom_path: String,
    // /// optional path to a bootrom file
    // #[argh(option, short = 'b')]
    // pub bootrom_path: Option<String>,
    // /// optional mode to run the emulator in (DMG, CGB, AUTO)
    // #[argh(option, short = 'm')]
    // pub mode: Option<String>,
}