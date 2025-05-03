#ifndef MAGENBOY_H
#define MAGENBOY_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stddef.h>
#include <stdint.h>

// Initialize the GameBoy instance.
//   rom: pointer to ROM data
//   rom_size: size of ROM data in bytes
// Returns: a pointer to the statically allocated GameBoy instance.
void* magenboy_init(const char* rom, uint64_t rom_size);

// Cycle a frame for the given GameBoy instance.
//   ctx: pointer to a GameBoy instance returned by magenboy_init.
void magenboy_cycle_frame(void* ctx);

#ifdef __cplusplus
}
#endif

#endif // MAGENBOY_H