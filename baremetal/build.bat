cargo +nightly b -r -Z build-std=core
rust-objcopy ../target/armv7a-none-eabihf/release/baremetal -O binary kernel7.img