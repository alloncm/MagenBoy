# We are installing the Rust toolchain so the version does not matter
FROM rust:latest AS builder

# Nightly version - entered as a build argument
ARG NIGHTLY

RUN rustup toolchain install ${NIGHTLY}
RUN rustup +${NIGHTLY} component add rust-src

WORKDIR /magenboy

COPY . .

RUN cargo +${NIGHTLY} build --release --package magenboy_nx --target aarch64-nintendo-switch-freestanding \
    -Z build-std=core,compiler_builtins,alloc -Z build-std-features=compiler-builtins-mem

# Use the devkitpro/devkita64 image for the second stage
FROM devkitpro/devkita64 AS final

WORKDIR /magenboy_nx

COPY nx/Makefile ./
COPY nx/src ./src

# Copy the built Rust library from the builder stage
COPY --from=builder /magenboy/target/ target/

# Needs to be run as the same RUN statement since the shell session is not shared between RUN statements
# Without this the Makefile will not be able to find the version and authors
RUN export VERSION=$(cat target/aarch64-nintendo-switch-freestanding/release/version.txt) && \
    export AUTHORS=$(cat target/aarch64-nintendo-switch-freestanding/release/authors.txt) && \
    echo "Version: ${VERSION}, authors: ${AUTHORS}" && \
    make --always-make

# Export the built files using a scratch image, this is the best praactice for multi-stage builds
FROM scratch AS export
COPY --from=final /magenboy_nx/build/ /target/nx