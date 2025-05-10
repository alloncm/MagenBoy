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
    memset(framebuffer, 0, 1280 * 720 * sizeof(uint16_t));

    for (int y = 0; y < height; y++)
    {
        uint16_t* dest = framebuffer + (y * stride);
        const uint16_t* src = buffer + (y * width);
        memcpy(dest, src, width * sizeof(uint16_t));
    }

    framebufferEnd(&fb);
}

static PadState pad;

static uint64_t get_joycon_state()
{
    // Pad update is being called in the main loop
    return padGetButtonsDown(&pad);
}

#define SAMPLERATE (48000)
#define CHANNEL_COUNT (2)
#define BYTES_PER_SAMPLE (sizeof(int16_t))
#define AUDIO_DATA_SIZE ((SAMPLERATE * CHANNEL_COUNT * BYTES_PER_SAMPLE) / (60))

// buffer for audio must be aligned to 0x1000 bytes
#define BUFFER_ALIGNMENT (0x1000)
#define AUDIO_BUFFER_SIZE ((AUDIO_DATA_SIZE + (BUFFER_ALIGNMENT - 1)) & ~(BUFFER_ALIGNMENT - 1)) /*Aligned buffer size*/

static uint32_t *audio_work_buffer;
static int audio_work_data_offset = 0;
static uint32_t *audio_io_buffer;
static bool first_buffer = true;

static void audio_device_cb(const int16_t* buffer, int size)
{
    for (int i = 0; i < size; i++)
    {
        // Convert each sample to a dual channel sample
        audio_work_buffer[audio_work_data_offset++] = (buffer[i] << 16) | (buffer[i] & 0xFFFF); 
        if (audio_work_data_offset >= (AUDIO_DATA_SIZE / (CHANNEL_COUNT + BYTES_PER_SAMPLE)))
        {
            audio_work_data_offset = 0;
            AudioOutBuffer *released_buffer = NULL;
            u32 count = 0;
            // wait for last buffer to finish playing
            if (first_buffer)
            {
                first_buffer = false;
            }
            else
            {
                audoutWaitPlayFinish(&released_buffer, &count, UINT64_MAX);
            }

            // Copy data to io buffer
            memcpy(audio_io_buffer, audio_work_buffer, AUDIO_BUFFER_SIZE);

            // Submit new samples
            AudioOutBuffer audio_out_buffer = {
                .next = NULL,
                .buffer = audio_io_buffer,
                .buffer_size = AUDIO_BUFFER_SIZE,
                .data_size = AUDIO_DATA_SIZE,
                .data_offset = 0,
            };
            audoutAppendAudioOutBuffer(&audio_out_buffer);
        }
    }
}

static int intiailzie_audio_buffers()
{
    audio_work_buffer = aligned_alloc(BUFFER_ALIGNMENT, AUDIO_BUFFER_SIZE);
    if (audio_work_buffer == NULL)
    {
        printf("Failed to allocate audio work buffer.\n");
        return -1;
    }
    audio_io_buffer = aligned_alloc(BUFFER_ALIGNMENT, AUDIO_BUFFER_SIZE);
    if (audio_io_buffer == NULL)
    {
        printf("Failed to allocate audio io buffer.\n");
        free(audio_work_buffer);
        return -1;
    }

    memset(audio_work_buffer, 0, AUDIO_BUFFER_SIZE);
    memset(audio_work_buffer, 0, AUDIO_BUFFER_SIZE);

    return 0;
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

    if (intiailzie_audio_buffers() != 0)
    {
        printf("Failed to initialize audio.\n");
        goto fb_exit;
    }
    if (R_FAILED(audoutInitialize()))
    {
        printf("Failed to initialize audio.\n");
        goto audio_buffers_exit;
    }
    if (R_FAILED(audoutStartAudioOut()))
    {
        printf("Failed to start audio.\n");
        goto audio_exit;
    }

    // Read a rom file
    char* rom_buffer = NULL;
    long file_size = read_rom_buffer("roms/PokemonRed.gb", &rom_buffer);
    if (file_size < 0)
    {
        printf("Failed to read ROM file.\n");
        goto fb_exit;
    }

    void* ctx = magenboy_init(rom_buffer, file_size, render_buffer_cb, get_joycon_state, audio_device_cb, log_cb); // Initialize the GameBoy instance with no ROM

    // Main loop
    while (appletMainLoop())
    {
        padUpdate(&pad);
        u64 kDown = padGetButtonsDown(&pad);
        if (kDown & HidNpadButton_X)
            break; // break in order to return to hbmenu

        magenboy_cycle_frame(ctx);
    }

    // Deinitialize and clean up resources
    free(rom_buffer);
    audoutStopAudioOut();
audio_exit:
    audoutExit();
audio_buffers_exit:
    free(audio_work_buffer);
    free(audio_io_buffer);
fb_exit:
    framebufferClose(&fb);
link_exit:
    close(nxlink_fd);
scoket_exit:
    socketExit();
    return 0;
}
