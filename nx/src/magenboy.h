#ifndef MAGENBOY_H
#define MAGENBOY_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>

// Define a callback type for logging.
typedef void (*LogCallback)(const char* message, int len);

// Graphics callback for swapping buffers
typedef void (*SwapBufferCallback)(void);

// OpenGL loader callback function
typedef void* (*GlLoaderCallback)(const char* proc_name);

// Joypad callbacks
typedef uint64_t (*JoypadProviderCallback)(void);
typedef uint64_t (*PollJoypadProviderCallback)(void);

// Audio callback - receives stereo samples (int16_t pairs)
typedef void (*AudioDeviceCallback)(const int16_t* buffer, int size);

void magenboy_init_logger(LogCallback log_cb);

// Initialize the GameBoy instance.
//   rom: pointer to ROM data
//   rom_size: size of ROM data in bytes
//   gfx_cb: callback for swapping buffers
//   gl_loader_callback: OpenGL function loader
//   window_width: width of the display window
//   window_height: height of the display window
//   joypad_cb: callback for providing joypad state
//   poll_joypad_cb: callback for polling joypad state
//   audio_cb: callback for audio output
// Returns: a pointer to the GameBoy context instance.
void* magenboy_init(const char* rom, uint64_t rom_size, SwapBufferCallback gfx_cb, GlLoaderCallback gl_loader_callback,
    uint32_t window_width, uint32_t window_height, JoypadProviderCallback joypad_cb, PollJoypadProviderCallback poll_joypad_cb,
    AudioDeviceCallback audio_cb);

void magenboy_deinit(void* ctx);

// Trigger the ROM selection menu
//   gfx_cb: callback for swapping buffers
//   joypad_cb: callback for providing joypad state
//   poll_joypad_cb: callback for polling joypad state
//   gl_loader_callback: OpenGL function loader
//   window_width: width of the display window
//   window_height: height of the display window
//   roms: array of ROM file paths (C strings)
//   roms_count: number of ROMs in the array
// Returns: pointer to the selected ROM path (C string)
const char* magenboy_menu_trigger(SwapBufferCallback gfx_cb, JoypadProviderCallback joypad_cb,
    PollJoypadProviderCallback poll_joypad_cb, GlLoaderCallback gl_loader_callback,
    uint32_t window_width, uint32_t window_height, const char** roms, uint32_t roms_count);

// Trigger the pause menu
//   ctx: gb context to operate on
// Returns: menu option index as uint32_t
uint32_t magenboy_pause_trigger(void* ctx);

// Cycle a frame for the given GameBoy instance.
//   ctx: pointer to a GameBoy context returned by magenboy_init.
//   This function polls the joypad, cycles the emulation, and renders a frame.
void magenboy_cycle_frame(void* ctx);

void magenboy_get_sram(void* ctx, uint8_t** sram_buffer, size_t* sram_size);

#ifdef __cplusplus
}
#endif

#endif // MAGENBOY_H