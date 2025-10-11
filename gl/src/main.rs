mod render;
mod cli;
mod input;

use std::{ffi::CString, ptr::null};

use glfw_sys::*;
use magenboy_common::{logging::init_fern_logger_with_log_level, mbc_handler::initialize_mbc, read_bootrom, log};
use magenboy_core::{GameBoy, Mode};

use crate::{input::GlfwJoypadProvider, render::update_viewport_callback};

const DEFAULT_WINDOW_WIDTH: u32 = 800;
const DEFAULT_WINDOW_HEIGHT: u32 = 600;

struct DummyAudioDevice;
impl magenboy_core::AudioDevice for DummyAudioDevice{
    fn push_buffer(&mut self, _buffer:&[magenboy_core::apu::audio_device::StereoSample; magenboy_core::apu::audio_device::BUFFER_SIZE]) {}
}

fn main() {
    init_fern_logger_with_log_level(Some(log::LevelFilter::Info)).expect("Error initializing logger");
    let args: cli::CliArgs = argh::from_env();
    let mode: Option<Mode> = args.mode.map(|m| m
        .as_str()
        .try_into()
        .expect(format!("Error! mode cannot be: {}", m).as_str())
    );

    let bootrom = read_bootrom(args.bootrom_path);
    
    let mbc = initialize_mbc(&args.rom_path);
    
    unsafe {
        if glfwInit() == 0 {
            println!("Failed to initialize GLFW");
            return;
        }

        glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
        glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
        glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

        let header = std::format!("MagenBoy v{}", magenboy_common::VERSION);
        let header = CString::new(header).unwrap();
        let window = glfwCreateWindow(
            DEFAULT_WINDOW_WIDTH as i32,
            DEFAULT_WINDOW_HEIGHT as i32,
            header.as_ptr() as _,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );

        if window.is_null() {
            println!("Failed to create GLFW window");
            glfwTerminate();
            return;
        }

        glfwMakeContextCurrent(window);
        glfwSetFramebufferSizeCallback(window, Some(update_viewport_callback));
        
        if glfwGetCurrentContext().is_null() {
            println!("Failed to get current context");
            return;
        }
        
        gl::load_with(|s| {
            let name = CString::new(s).unwrap();
            match glfwGetProcAddress(name.as_ptr()) {
                Some(p) => p as *const _,
                None => null(),
            }
        });
        
        let renderer = render::GlRenderer::new(window);
        let joypad_provider = GlfwJoypadProvider::new(window);

        let mut gameboy = match bootrom {
            Some(b) => GameBoy::new_with_bootrom(mbc, joypad_provider, DummyAudioDevice, renderer, b),
            None => {
                let mode = mode.unwrap_or_else(|| mbc.detect_preferred_mode());
                GameBoy::new_with_mode(
                    mbc,
                    joypad_provider,
                    DummyAudioDevice,
                    renderer,
                    mode
                )
            }
        };

        while glfwWindowShouldClose(window) == 0 {
            process_input(window);

            gameboy.cycle_frame();

            glfwPollEvents();
        }

        glfwTerminate();
    }
}

fn process_input(window: *mut GLFWwindow) {
    unsafe {
        if glfwGetKey(window, GLFW_KEY_ESCAPE) == GLFW_PRESS {
            glfwSetWindowShouldClose(window, 1);
        }
    }
}