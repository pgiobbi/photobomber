# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.85.1
ARG APP_NAME=photobomb-backend

################################################################################
# Create a stage for building the application.
FROM --platform=linux/arm64 rust:${RUST_VERSION}-bullseye AS build
ARG APP_NAME
WORKDIR /app

# Install build dependencies.
RUN apt-get update && apt-get install -y \
    clang \
    lld \
    libssl-dev \
    pkg-config \
    git \
    && rm -rf /var/lib/apt/lists/*

# Copy source code and build the application.
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --locked --release --target aarch64-unknown-linux-gnu
RUN cp ./target/aarch64-unknown-linux-gnu/release/${APP_NAME} /bin/server

################################################################################
# Create a new stage for running the application.
FROM --platform=linux/arm64 debian:bullseye-slim AS final

# Install runtime dependencies.
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-privileged user.
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the executable from the build stage.
COPY --from=build /bin/server /bin/

# Expose the port.
EXPOSE 3002

# Run the application.
CMD ["/bin/server"]
