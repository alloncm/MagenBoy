#include <dirent.h>
#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

// Include the main libnx system header, for Switch development
#include <switch.h>
#include <EGL/egl.h>

// Include magenboy header
#include "magenboy.h"

#define MIN(X, Y) (((X) < (Y)) ? (X) : (Y))

static void log_cb(const char* message, int len) {
    fwrite(message, 1, len, stdout);
}

static long read_rom_buffer(const char* path, char** out_rom_buffer) {
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

    *out_rom_buffer = (char*)malloc(size);
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

static PadState pad;

static uint64_t get_joycon_state() {
    padUpdate(&pad);
    return padGetButtons(&pad);
}

static uint64_t poll_until_joycon_pressed() {
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

static void audio_device_cb(const int16_t* buffer, int size) {
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

static int intiailzie_audio_buffers() {
    audio_work_buffer = (int16_t*)aligned_alloc(BUFFER_ALIGNMENT, AUDIO_BUFFER_SIZE);
    if (audio_work_buffer == NULL) {
        printf("Failed to allocate audio work buffer.\n");
        return -1;
    }
    audio_io_buffer = (int16_t*)aligned_alloc(BUFFER_ALIGNMENT, AUDIO_BUFFER_SIZE);
    if (audio_io_buffer == NULL) {
        printf("Failed to allocate audio io buffer.\n");
        free(audio_work_buffer);
        return -1;
    }

    memset(audio_work_buffer, 0, AUDIO_BUFFER_SIZE);
    memset(audio_io_buffer, 0, AUDIO_BUFFER_SIZE);

    return 0;
}

static EGLDisplay egl_display;
static EGLContext egl_context;
static EGLSurface egl_surface;

static void swap_buffers_cb() {
    eglSwapBuffers(egl_display, egl_surface);
}

static int initialize_egl(NWindow *win) {
    egl_display = eglGetDisplay(EGL_DEFAULT_DISPLAY);
    if (!egl_display) {
        printf("Could not connect to display! error: %d\n", eglGetError());
        goto err;
    }

    eglInitialize(egl_display, NULL, NULL);

    if (EGL_FALSE == eglBindAPI(EGL_OPENGL_API)) {
        printf("Could not set API! error: %d\n", eglGetError());
        goto err_free_display;
    }

    EGLConfig config;
    EGLint num_configs;
    static const EGLint framebuffer_attributes[] = {
        EGL_RENDERABLE_TYPE, EGL_OPENGL_BIT,
        EGL_RED_SIZE, 5,
        EGL_GREEN_SIZE, 6,
        EGL_BLUE_SIZE, 5,
        EGL_NONE
    };
    eglChooseConfig(egl_display, framebuffer_attributes, &config, 1, &num_configs);
    if (0 == num_configs) {
        printf("No config found! error: %d\n", eglGetError());
        goto err_free_display;
    }

    egl_surface = eglCreateWindowSurface(egl_display, config, (EGLNativeWindowType)win, NULL);
    if (!egl_surface) {
        printf("Surface creation failed! error: %d\n", eglGetError());
        goto err_free_display;
    }

    static const EGLint context_attriubtes[] = {
        EGL_CONTEXT_OPENGL_PROFILE_MASK, EGL_CONTEXT_OPENGL_CORE_PROFILE_BIT,
        EGL_CONTEXT_MAJOR_VERSION, 3,
        EGL_CONTEXT_MINOR_VERSION, 3,
        EGL_NONE
    };
    egl_context = eglCreateContext(egl_display, config, EGL_NO_CONTEXT, context_attriubtes);
    if (!egl_context) {
        printf("Context creation failed! error: %d\n", eglGetError());
        goto err_free_surface;
    }

    eglMakeCurrent(egl_display, egl_surface, egl_surface, egl_context);

    // Disable VSYNC
    eglSwapInterval(egl_display, 0);

    return 0;

err_free_surface:
    eglDestroySurface(egl_display, egl_context);
    egl_surface = NULL;
err_free_display:
    eglTerminate(egl_display);
    egl_display = NULL;
err:
    return -1;
}

static void deinit_egl()
{
    if (egl_display)
    {
        eglMakeCurrent(egl_display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
        if (egl_context)
        {
            eglDestroyContext(egl_display, egl_context);
            egl_context = NULL;
        }
        if (egl_surface)
        {
            eglDestroySurface(egl_display, egl_surface);
            egl_surface = NULL;
        }
        eglTerminate(egl_display);
        egl_display = NULL;
    }
}

static void get_timespec(struct timespec* ts) {
    clock_gettime(CLOCK_MONOTONIC, ts);
}

static int has_gb_extension(const char* filename) {
    const char* ext = strrchr(filename, '.');
    if (ext && (strcmp(ext, ".gb") == 0 || strcmp(ext, ".gbc") == 0)) {
        return 1;
    }
    return 0;
}

static int read_dir_filenames(const char* directory_path, char** file_list, size_t max_filename_size, size_t max_files) {
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

static int try_load_sram(const char* filepath, u8** sram_buffer, size_t* sram_size) {
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

static void save_sram(const char* filepath, const u8* sram_buffer, size_t sram_size) {
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

int main(int argc, char* argv[]) {
    if (socketInitializeDefault() != 0) {
        printf("Failed to initialize socket driver.\n");
        return -1;
    }
    int nxlink_fd = nxlinkStdio();
    if (nxlink_fd < 0) {
        printf("Failed to initialize NXLink: %d.\n", errno);
        socketExit();
        goto link_exit;
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

    if (0 != initialize_egl(win)) {
        printf("Failed to init egl");
        goto link_exit;
    }

    if (intiailzie_audio_buffers() != 0) {
        printf("Failed to initialize audio.\n");
        goto egl_exit;
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

    const char* filepath = magenboy_menu_trigger(
        swap_buffers_cb,
        get_joycon_state,
        poll_until_joycon_pressed,
        (GlLoaderCallback)eglGetProcAddress,
        win_width,
        win_height,
        (const char**)roms,
        count
    );
    if (filepath == NULL) {
        printf("Failed to trigger ROM menu.\n");
        goto egl_exit;
    }

    // Read a rom file
    char* rom_buffer = NULL;
    long file_size = read_rom_buffer(filepath, &rom_buffer);
    if (file_size < 0) {
        printf("Failed to read ROM file.\n");
        goto egl_exit;
    }

    u8* found_sram_buffer = NULL;
    size_t found_sram_size = 0;
    int found_sram = try_load_sram(filepath, &found_sram_buffer, &found_sram_size);

    void* ctx = magenboy_init(
        rom_buffer,
        file_size,
        swap_buffers_cb,
        (GlLoaderCallback)eglGetProcAddress,
        win_width,
        win_height,
        get_joycon_state,
        poll_until_joycon_pressed,
        audio_device_cb
    );

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
        // No need to update as the joypad called is polling the state
        u64 kDown = padGetButtons(&pad);
        if ((kDown & HidNpadButton_L) != 0 && (kDown & HidNpadButton_R) != 0) {
            int shutdown = 0;
            int menu_option = magenboy_pause_trigger(ctx);
            switch (menu_option) {
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
    printf("Close audio\n");
audio_buffers_exit:
    free(audio_work_buffer);
    free(audio_io_buffer);
egl_exit:
    deinit_egl();
    printf("Close egl\n");
link_exit:
    if (nxlink_fd > 0) {
        close(nxlink_fd);
        printf("CLosed nx link\n");
    }
scoket_exit:
    if (nxlink_fd > 0) {
        socketExit();
        printf("CLosed sockets\n");
    }
    printf("Closing MagenBoy NX port\n");
    return EXIT_SUCCESS;
}
