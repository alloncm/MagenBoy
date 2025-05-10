// Include the most common headers from the C standard library
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>

// Include the main libnx system header, for Switch development
#include <switch.h>

// Include magenboy header
#include "magenboy.h"

static void log_cb(const char* message, int len)
{
    fwrite(message, 1, len, stdout);
}

static const long read_rom_buffer(const char* path, char** out_rom_buffer)
{
    long return_value = -1;
    *out_rom_buffer = NULL;

    FILE* file = fopen(path, "rb");
    if (!file)
    {
        perror("Failed to open ROM file");
        return return_value;
    }

    if (fseek(file, 0, SEEK_END) != 0)
    {
        perror("Failed to seek to end of ROM file");
        goto exit_file;
    }
    long size = ftell(file);
    rewind(file);

    *out_rom_buffer = (char*)malloc(size);
    if (!out_rom_buffer)
    {
        perror("Failed to allocate memory for ROM");
        goto exit_file;
    }

    if (fread(*out_rom_buffer, 1, size, file) != size)
    {
        perror("Failed to read ROM file");    
        free(*out_rom_buffer);
        *out_rom_buffer = NULL;
    }

    return_value = size;

exit_file:
    fclose(file);
    return return_value;
}

static Framebuffer fb;

static void render_buffer_cb(const uint16_t* buffer, int width, int height)
{
    u32 stride;
    uint16_t* framebuffer = (uint16_t*)framebufferBegin(&fb, &stride);
    stride /= sizeof(uint16_t);
    printf("width, height, stride: %d %d %d\n", width, height, stride);
    memset(framebuffer, 0, 1280 * 720 * sizeof(uint16_t));

    for (int y = 0; y < height; y++)
    {
        uint16_t* dest = framebuffer + (y * stride);
        const uint16_t* src = buffer + (y * width);
        memcpy(dest, src, width * sizeof(uint16_t));
    }

    framebufferEnd(&fb);
}

int main(int argc, char* argv[])
{
    if (socketInitializeDefault() != 0)
    {
        printf("Failed to initialize socket driver.\n");
        return -1;
    }
    int nxlink_fd = nxlinkStdio();
    if (nxlink_fd < 0)
    {
        printf("Failed to initialize NXLink: %d.\n", errno);
        goto scoket_exit;
    }

    // Configure our supported input layout: a single player with standard controller styles
    padConfigureInput(1, HidNpadStyleSet_NpadStandard);

    // Initialize the default gamepad (which reads handheld mode inputs as well as the first connected controller)
    PadState pad;
    padInitializeDefault(&pad);

    // Retrieve the default window
    NWindow* win = nwindowGetDefault();

    // Initialize the framebuffer
    if (R_FAILED(framebufferCreate(&fb, win, 1280, 720, PIXEL_FORMAT_RGB_565, 2))) 
    {
        printf("Failed to create framebuffer.\n");
        goto link_exit;
    }

    if (R_FAILED(framebufferMakeLinear(&fb)))
    {
        printf("Failed to make framebuffer linear.\n");
        goto fb_exit;
    }

    // Read a rom file
    char* rom_buffer = NULL;
    long file_size = read_rom_buffer("roms/PokemonRed.gb", &rom_buffer);
    if (file_size < 0)
    {
        printf("Failed to read ROM file.\n");
        goto fb_exit;
    }

    void* ctx = magenboy_init(rom_buffer, file_size, render_buffer_cb, log_cb); // Initialize the GameBoy instance with no ROM

    // Main loop
    while (appletMainLoop())
    {
        // Scan the gamepad. This should be done once for each frame
        padUpdate(&pad);

        // padGetButtonsDown returns the set of buttons that have been
        // newly pressed in this frame compared to the previous one
        u64 kDown = padGetButtonsDown(&pad);

        if (kDown & HidNpadButton_Plus)
            break; // break in order to return to hbmenu

        // Your code goes here
        magenboy_cycle_frame(ctx);
    }

    // Deinitialize and clean up resources
    free(rom_buffer);
fb_exit:
    framebufferClose(&fb);
link_exit:
    close(nxlink_fd);
scoket_exit:
    socketExit();
    return 0;
}
