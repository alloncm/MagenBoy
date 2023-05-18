# armv7a-none-eabihf is not supported automacticaly by rust so the nightly toolchain is neccessary to build the core library
cargo +nightly b -r -Z build-std=core
rust-objcopy ../target/armv7a-none-eabihf/release/baremetal -O binary kernel7.img