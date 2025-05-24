# We are installing the Rust toolchain so the version does not matter
FROM rust:latest AS builder

# Nightly version - entered as a build argument
ARG NIGHTLY

# Build the Rust crate for the Nintendo Switch target
RUN rustup toolchain install ${NIGHTLY}
RUN rustup +${NIGHTLY} component add rust-src

# Set the working directory
WORKDIR /magenboy

# Copy source files
COPY . .

RUN cargo +${NIGHTLY} build --release --package magenboy_nx --target aarch64-nintendo-switch-freestanding \
    -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem

# Use the devkitpro/devkita64 image for the second stage
FROM devkitpro/devkita64 AS final

# Set the working directory for the C code
WORKDIR /magenboy_nx

# Copy the C source files and Makefile
COPY nx/Makefile ./
COPY nx/src ./src

# Copy the built Rust library from the builder stage
COPY --from=builder /magenboy/target/ ./target/

RUN make --always-make

FROM scratch AS export
COPY --from=final /magenboy_nx/build/ /build