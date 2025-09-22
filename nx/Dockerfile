FROM devkitpro/devkita64 AS final

WORKDIR /magenboy_nx

COPY nx/Makefile ./
# C code, copies also the Rust code cause im lazy to filter
COPY nx/src ./src
# aarch64-nintendo-switch-freestanding Rust artifacts
COPY target/aarch64-nintendo-switch-freestanding/ target/

# Needs to be run as the same RUN statement since the shell session is not shared between RUN statements
# Without this the Makefile will not be able to find the version and authors
RUN export VERSION=$(cat target/release/version.txt) && \
    export AUTHORS=$(cat target/release/authors.txt) && \
    make --always-make

# Export the built files using a scratch image, this is the best practice for multi-stage builds
FROM scratch AS export
COPY --from=final /magenboy_nx/build/ /target/nx