#include <dirent.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

// Include the main libnx system header, for Switch development
#include <switch.h>

// Include magenboy header
#include "magenboy.h"

#define MIN(X, Y) (((X) < (Y)) ? (X) : (Y))

static void log_cb(const char* message, int len)
{
    fwrite(message, 1, len, stdout);
}

static long read_rom_buffer(const char* path, u8** out_rom_buffer)
{
    long return_value = -1;
    *out_rom_buffer = NULL;

    FILE* file = fopen(path, "rb");
    if (!file) {
        perror("Failed to open ROM file");
        return return_value;
    }

    if (fseek(file, 0, SEEK_END) != 0) {
        perror("Failed to seek to end of ROM file");
        goto exit_file;
    }
    long size = ftell(file);
    rewind(file);

    *out_rom_buffer = (u8*)malloc(size);
    if (!out_rom_buffer) {
        perror("Failed to allocate memory for ROM");
        goto exit_file;
    }

    if (fread(*out_rom_buffer, 1, size, file) != size) {
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

static void render_buffer_cb(const uint16_t* buffer)
{
    u32 stride;
    uint16_t* framebuffer = (uint16_t*)framebufferBegin(&fb, &stride);
    stride /= sizeof(uint16_t);

    u32 gb_width, gb_height;
    magenboy_get_dimensions(&gb_width, &gb_height);

    u32 frame_initial_width = (stride - gb_width) / 2;

    for (int y = 0; y < gb_height; y++) {
        uint16_t* dest = framebuffer + (y * stride) + frame_initial_width;
        const uint16_t* src = buffer + (y * gb_width);
        memcpy(dest, src, gb_width * sizeof(uint16_t));
    }

    framebufferEnd(&fb);
}

static PadState pad;

static uint64_t get_joycon_state()
{
    padUpdate(&pad);
    return padGetButtons(&pad);
}

static uint64_t poll_until_joycon_pressed()
{
    while (1) {
        padUpdate(&pad);
        u64 buttons = padGetButtonsDown(&pad);
        if (buttons != 0) {
            return buttons;
        }

        svcSleepThread(10000000ULL); // 100ms in nanoseconds
    }
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

static int16_t* audio_work_buffer;
static int audio_work_data_offset = 0;
static int16_t* audio_io_buffer;

static void audio_device_cb(const int16_t* buffer, int size)
{
    int transfer_size = MIN(size, (AUDIO_DATA_SIZE / BYTES_PER_SAMPLE) - audio_work_data_offset);
    memcpy(audio_work_buffer + audio_work_data_offset, buffer, transfer_size * BYTES_PER_SAMPLE);
    audio_work_data_offset += transfer_size;

    if (audio_work_data_offset >= (AUDIO_DATA_SIZE / BYTES_PER_SAMPLE)) {
        audio_work_data_offset = 0;

        // wait for last buffer to finish playing
        AudioOutBuffer* released_buffer = NULL;
        u32 count = 0;
        audoutWaitPlayFinish(&released_buffer, &count, UINT64_MAX);

        // Copy data to buffer
        if (released_buffer) {
            memcpy(released_buffer->buffer, audio_work_buffer, released_buffer->data_size);
        }

        // Submit new samples
        audoutAppendAudioOutBuffer(released_buffer);
    }

    if (transfer_size < size) {
        int remaining_size = size - transfer_size;
        memcpy(audio_work_buffer + audio_work_data_offset, buffer + transfer_size, remaining_size * BYTES_PER_SAMPLE);
        audio_work_data_offset += remaining_size;
    }
}

static int intiailzie_audio_buffers()
{
    audio_work_buffer = aligned_alloc(BUFFER_ALIGNMENT, AUDIO_BUFFER_SIZE);
    if (audio_work_buffer == NULL) {
        printf("Failed to allocate audio work buffer.\n");
        return -1;
    }
    audio_io_buffer = aligned_alloc(BUFFER_ALIGNMENT, AUDIO_BUFFER_SIZE);
    if (audio_io_buffer == NULL) {
        printf("Failed to allocate audio io buffer.\n");
        free(audio_work_buffer);
        return -1;
    }

    memset(audio_work_buffer, 0, AUDIO_BUFFER_SIZE);
    memset(audio_work_buffer, 0, AUDIO_BUFFER_SIZE);

    return 0;
}

static void get_timespec(struct timespec* ts)
{
    clock_gettime(CLOCK_MONOTONIC, ts);
}

static int has_gb_extension(const char* filename)
{
    const char* ext = strrchr(filename, '.');
    if (ext && (strcmp(ext, ".gb") == 0 || strcmp(ext, ".gbc") == 0)) {
        return 1;
    }
    return 0;
}

static int read_dir_filenames(const char* directory_path, char** file_list, size_t max_filename_size, size_t max_files)
{
    struct dirent* entry;
    DIR* dir = opendir(directory_path);

    if (dir == NULL) {
        perror("Failed to open directory");
        return -1;
    }

    printf("Files in directory '%s':\n", directory_path);
    int counter = 0;
    while ((entry = readdir(dir)) != NULL) {
        if (has_gb_extension(entry->d_name) != 0) {
            printf("%s\n", entry->d_name);

            if (counter < max_files) {
                snprintf(file_list[counter], max_filename_size, "%s/%s", directory_path, entry->d_name);
                counter++;
            } else {
                printf("Maximum number of files reached.\n");
                break;
            }
        }
    }

    closedir(dir);
    return counter;
}

#define MAX_ROMS (30)
#define MAX_FILENAME_SIZE (300)

static int try_load_sram(const char* filepath, u8** sram_buffer, size_t* sram_size)
{
    int status = 0;
    char sram_path[MAX_FILENAME_SIZE];
    snprintf(sram_path, sizeof(sram_path), "%s.sram", filepath);

    FILE* file = fopen(sram_path, "rb");
    if (!file) {
        perror("Failed to open SRAM file");
        return -1;
    }

    fseek(file, 0, SEEK_END);
    *sram_size = ftell(file);
    rewind(file);

    *sram_buffer = (u8*)malloc(*sram_size);
    if (!*sram_buffer) {
        perror("Failed to allocate memory for SRAM");
        status = -1;
        goto exit;
    }

    if (fread(*sram_buffer, 1, *sram_size, file) != *sram_size) {
        perror("Failed to read SRAM file");
        free(*sram_buffer);
        status = -1;
        goto exit;
    }

exit:
    fclose(file);
    return status;
}

static void save_sram(const char* filepath, const u8* sram_buffer, size_t sram_size)
{
    char sram_path[MAX_FILENAME_SIZE];
    snprintf(sram_path, sizeof(sram_path), "%s.sram", filepath);

    FILE* file = fopen(sram_path, "wb");
    if (!file) {
        perror("Failed to open SRAM file for writing");
        return;
    }

    if (fwrite(sram_buffer, 1, sram_size, file) != sram_size) {
        perror("Failed to write SRAM data");
    }

    fclose(file);
}

int main(int argc, char* argv[])
{
    if (socketInitializeDefault() != 0) {
        printf("Failed to initialize socket driver.\n");
        return -1;
    }
    int nxlink_fd = nxlinkStdio();
    if (nxlink_fd < 0) {
        printf("Failed to initialize NXLink: %d.\n", errno);
        socketExit();
    }

    // Configure our supported input layout: a single player with standard controller styles
    padConfigureInput(1, HidNpadStyleSet_NpadStandard);

    // Initialize the default gamepad (which reads handheld mode inputs as well as the first connected controller)
    padInitializeDefault(&pad);

    // Retrieve the default window
    NWindow* win = nwindowGetDefault();

    u32 win_width, win_height;
    if (R_FAILED(nwindowGetDimensions(win, &win_width, &win_height))) {
        printf("Failed to get window dimensions.\n");
        goto scoket_exit;
    }

    u32 gb_wifth, gb_height;
    magenboy_get_dimensions(&gb_wifth, &gb_height);

    // Adjusting the framebuffer width to match the window width in order to let the switch scale the image
    float width_scale_ratio = (float)win_height / (float)gb_height;
    u32 frame_width = (u32)(gb_wifth * (float)win_width / (float)(gb_wifth * width_scale_ratio));

    // Initialize the framebuffer
    if (R_FAILED(framebufferCreate(&fb, win, frame_width, gb_height, PIXEL_FORMAT_RGB_565, 2))) {
        printf("Failed to create framebuffer.\n");
        goto link_exit;
    }

    if (R_FAILED(framebufferMakeLinear(&fb))) {
        printf("Failed to make framebuffer linear.\n");
        goto fb_exit;
    }

    if (intiailzie_audio_buffers() != 0) {
        printf("Failed to initialize audio.\n");
        goto fb_exit;
    }
    if (R_FAILED(audoutInitialize())) {
        printf("Failed to initialize audio.\n");
        goto audio_buffers_exit;
    }
    if (R_FAILED(audoutStartAudioOut())) {
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

    magenboy_init_logger(log_cb);

    // Asks the user to select a ROM file
    char** roms = malloc(MAX_ROMS * sizeof(char*));
    for (int i = 0; i < MAX_ROMS; i++) {
        roms[i] = malloc(MAX_FILENAME_SIZE);
    }

restart:
    int count = read_dir_filenames("roms", roms, MAX_FILENAME_SIZE, MAX_ROMS);

    const char* filepath = magenboy_menu_trigger(render_buffer_cb, get_joycon_state, poll_until_joycon_pressed, (const char**)roms, count);
    if (filepath == NULL) {
        printf("Failed to trigger ROM menu.\n");
        goto fb_exit;
    }

    // Read a rom file
    u8* rom_buffer = NULL;
    long file_size = read_rom_buffer(filepath, &rom_buffer);
    if (file_size < 0) {
        printf("Failed to read ROM file.\n");
        goto fb_exit;
    }

    u8* found_sram_buffer = NULL;
    size_t found_sram_size = 0;
    int found_sram = try_load_sram(filepath, &found_sram_buffer, &found_sram_size);

    void* ctx = magenboy_init(rom_buffer, file_size, render_buffer_cb, get_joycon_state, poll_until_joycon_pressed, audio_device_cb);

    u8* sram_buffer = NULL;
    size_t sram_size = 0;
    magenboy_get_sram(ctx, &sram_buffer, &sram_size);

    if (found_sram == 0 && sram_size == found_sram_size) {
        memcpy(sram_buffer, found_sram_buffer, sram_size);
        printf("Loaded SRAM from file: %s.sram\n", filepath);
    }

    // FPS measurement variables
    struct timespec start_time, end_time;
    int frame_count = 0;
    double elapsed_time = 0.0;

    get_timespec(&start_time);

    // Main loop
    while (appletMainLoop()) {
        padUpdate(&pad);
        u64 kDown = padGetButtonsDown(&pad);
        if (kDown & HidNpadButton_X) {
            int shutdown = 0;
            switch (magenboy_pause_trigger(render_buffer_cb, get_joycon_state, poll_until_joycon_pressed)) {
            case 0: // Resume
                break;
            case 1: // Restart
                printf("Restarting\n");
                goto restart;
            case 2: // Shutdon
                printf("Shutting down\n");
                shutdown = 1;
                break;
            }
            if (shutdown) {
                break; // Exit the main loop
            }
        }

        magenboy_cycle_frame(ctx);

        // FPS calculation
        frame_count++;
        get_timespec(&end_time);
        elapsed_time = (end_time.tv_sec - start_time.tv_sec) + (end_time.tv_nsec - start_time.tv_nsec) / 1e9;

        // Print FPS every second
        if (elapsed_time >= 1.0) {
            printf("FPS: %d\n", frame_count);
            frame_count = 0;
            get_timespec(&start_time);
        }
    }

    save_sram(filepath, sram_buffer, sram_size);

    // Deinitialize and clean up resources
    magenboy_deinit(ctx);
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
    if (nxlink_fd > 0) {
        close(nxlink_fd);
    }
scoket_exit:
    if (nxlink_fd > 0) {
        socketExit();
    }
    return 0;
}
