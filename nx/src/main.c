// Include the most common headers from the C standard library
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include <time.h> 

// Include the main libnx system header, for Switch development
#include <switch.h>

// Include magenboy header
#include "magenboy.h"

#define MIN(X, Y) (((X) < (Y)) ? (X) : (Y))

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
// For sone reason multplying by 40 makes the best audio latency (60 for example makes audoutWaitPlayFinish to block for a long time)
// causing frame drops and audio glitches
#define AUDIO_DATA_SIZE ((SAMPLERATE * CHANNEL_COUNT * BYTES_PER_SAMPLE) / (40))

// buffer for audio must be aligned to 0x1000 bytes
#define BUFFER_ALIGNMENT (0x1000)
#define AUDIO_BUFFER_SIZE ((AUDIO_DATA_SIZE + (BUFFER_ALIGNMENT - 1)) & ~(BUFFER_ALIGNMENT - 1)) /*Aligned buffer size*/

static int16_t *audio_work_buffer;
static int audio_work_data_offset = 0;
static int16_t *audio_io_buffer;

static void audio_device_cb(const int16_t* buffer, int size)
{
    int transfer_size = MIN(size, (AUDIO_DATA_SIZE / BYTES_PER_SAMPLE) - audio_work_data_offset);
    memcpy(audio_work_buffer + audio_work_data_offset, buffer, transfer_size * BYTES_PER_SAMPLE);
    audio_work_data_offset += transfer_size;
    
    if (audio_work_data_offset >= (AUDIO_DATA_SIZE / BYTES_PER_SAMPLE))
    {
        audio_work_data_offset = 0;
        
        // wait for last buffer to finish playing
        AudioOutBuffer *released_buffer = NULL;
        u32 count = 0;
        audoutWaitPlayFinish(&released_buffer, &count, UINT64_MAX);

        // Copy data to buffer
        if (released_buffer)
        {
            memcpy(released_buffer->buffer, audio_work_buffer, released_buffer->data_size);
        }

        // Submit new samples
        audoutAppendAudioOutBuffer(released_buffer);
    }

    if (transfer_size < size)
    {
        int remaining_size = size - transfer_size;
        memcpy(audio_work_buffer + audio_work_data_offset, buffer + transfer_size, remaining_size * BYTES_PER_SAMPLE);
        audio_work_data_offset += remaining_size;
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

    // Initialize the audio output buffer

    AudioOutBuffer audio_out_buffer = {
        .buffer = audio_io_buffer,
        .buffer_size = AUDIO_BUFFER_SIZE,
        .data_size = AUDIO_DATA_SIZE,
        .data_offset = 0,
        .next = NULL,
    };

    audoutAppendAudioOutBuffer(&audio_out_buffer);

    // Read a rom file
    char* rom_buffer = NULL;
    long file_size = read_rom_buffer("roms/PokemonRed.gb", &rom_buffer);
    if (file_size < 0)
    {
        printf("Failed to read ROM file.\n");
        goto fb_exit;
    }

    void* ctx = magenboy_init(rom_buffer, file_size, render_buffer_cb, get_joycon_state, audio_device_cb, log_cb); // Initialize the GameBoy instance with no ROM

    // FPS measurement variables
    struct timespec start_time, end_time;
    int frame_count = 0;
    double elapsed_time = 0.0;

    clock_gettime(CLOCK_MONOTONIC, &start_time);

    printf("Sample rate: %d\n", audoutGetSampleRate());
    printf("Channel count: %d\n", audoutGetChannelCount());
    printf("PCM format: %d\n", audoutGetPcmFormat());
    printf("Device state: %d\n", audoutGetDeviceState());

    // Main loop
    while (appletMainLoop())
    {
        padUpdate(&pad);
        u64 kDown = padGetButtonsDown(&pad);
        if (kDown & HidNpadButton_X)
            break; // break in order to return to hbmenu

        magenboy_cycle_frame(ctx);

        // FPS calculation
        frame_count++;
        clock_gettime(CLOCK_MONOTONIC, &end_time);
        elapsed_time = (end_time.tv_sec - start_time.tv_sec) + 
                       (end_time.tv_nsec - start_time.tv_nsec) / 1e9;

        if (elapsed_time >= 1.0) // Print FPS every second
        {
            printf("FPS: %d\n", frame_count);
            frame_count = 0;
            clock_gettime(CLOCK_MONOTONIC, &start_time);
        }
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
