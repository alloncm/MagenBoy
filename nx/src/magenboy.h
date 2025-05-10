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
typedef void (*GfxDeviceCallback)(const uint16_t* buffer, int width, int height);

// Initialize the GameBoy instance.
//   rom: pointer to ROM data
//   rom_size: size of ROM data in bytes
// Returns: a pointer to the statically allocated GameBoy instance.
void* magenboy_init(const char* rom, uint64_t rom_size, GfxDeviceCallback gfx_cb, LogCallback log_cb);

// Cycle a frame for the given GameBoy instance.
//   ctx: pointer to a GameBoy instance returned by magenboy_init.
void magenboy_cycle_frame(void* ctx);

#ifdef __cplusplus
}
#endif

#endif // MAGENBOY_H