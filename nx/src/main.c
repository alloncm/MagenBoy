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

static const char rom[0x1000] = {0};

// Main program entrypoint
int main(int argc, char* argv[])
{
    // This example uses a text console, as a simple way to output text to the screen.
    // If you want to write a software-rendered graphics application,
    //   take a look at the graphics/simplegfx example, which uses the libnx Framebuffer API instead.
    // If on the other hand you want to write an OpenGL based application,
    //   take a look at the graphics/opengl set of examples, which uses EGL instead.
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

    // Other initialization goes here. As a demonstration, we print hello world.
    printf("Hello World!\n");

    magenboy_init(rom, 0x1000); // Initialize the GameBoy instance with no ROM

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
    }

    // Deinitialize and clean up resources
    close(nxlink_fd);
scoket_exit:
    socketExit();
    return 0;
}
