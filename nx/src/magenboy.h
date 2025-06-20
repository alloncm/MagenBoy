#ifndef MAGENBOY_H
#define MAGENBOY_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>

// Define a callback type for logging.
// Adjust the signature as needed.
typedef void (*LogCallback)(const char* message, int len);
typedef void (*GfxDeviceCallback)(const uint16_t* buffer);
typedef uint64_t (*JoypadDeviceCallback)();
typedef uint64_t (*PollJoypadDeviceCallback)();
typedef void (*AudioDeviceCallback)(const int16_t* buffer, int size);

void magenboy_init_logger(LogCallback log_cb);

// Initialize the GameBoy instance.
//   rom: pointer to ROM data
//   rom_size: size of ROM data in bytes
// Returns: a pointer to the statically allocated GameBoy instance.
void* magenboy_init(const uint8_t* rom, uint64_t rom_size, GfxDeviceCallback gfx_cb, JoypadDeviceCallback joypad_cb, PollJoypadDeviceCallback poll_cb,
    AudioDeviceCallback audio_cb);

void magenboy_deinit(void* ctx);

const char* magenboy_menu_trigger(GfxDeviceCallback gfx_cb, JoypadDeviceCallback joypad_cb, PollJoypadDeviceCallback poll_cb, const char** roms, uint32_t roms_count);

const uint32_t magenboy_pause_trigger(GfxDeviceCallback gfx_cb, JoypadDeviceCallback joypad_cb, PollJoypadDeviceCallback poll_cb);

// Cycle a frame for the given GameBoy instance.
//   ctx: pointer to a GameBoy instance returned by magenboy_init.
void magenboy_cycle_frame(void* ctx);

// Get the GB display dimensions.
void magenboy_get_dimensions(uint32_t* width, uint32_t* height);

void magenboy_get_sram(void* ctx, uint8_t** sram_buffer, size_t* sram_size);

#ifdef __cplusplus
}
#endif

#endif // MAGENBOY_H