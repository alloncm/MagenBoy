# rusty_gfx
A simple minimalist and easy to use wrapper around rust sdl2 bindings

I created this wrapper in order to use sdl in my gameboy emulator but in a more convient and easy way.

## how to build 

Currently this crate is not on crates.io so I recommend to add it as a git submodule
In order to build it you need to install cmake (sdl2 needs it to build itself) and then just hit the good old ```cargo build```

## How to use
Initialize the lib:
```rust
use rusty_gfx::{
    event_handler::EventHandler,
    graphics::Graphics,
    initializer::Initializer
};

    let gfx_initializer: Initializer = Initializer::new();
    let mut graphics: Graphics = gfx_initializer.init_graphics("app_name", 800, 600);
    let mut event_handler: EventHandler = gfx_initializer.init_event_handler();
```

Use the lib 
```rust
while event_handler.handle_events() {
        graphics.clear();
        /*logic rendering code goes here*/
        
        //you can put one pixel to the screen with: x, y, r, g, b (in the future there will be also and alpha option)
        graphics.put_pixel(x,y,r,g,b);
        
        //you can draw a surface which is just a bunch of pixels together with: x, y, &Surface
        graphics.draw_surface(x,y,&surface);
        
        /*your logic rendering code stops here*/
        graphics.update();
    }
```
