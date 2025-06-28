# Stage 1: Build the application
FROM rust:latest AS builder

WORKDIR /usr/src/blt
COPY . .

# Build the release binary
RUN cargo build --release

# Stage 2: Create the runtime image
FROM debian:slim

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/blt/target/release/blt /usr/local/bin/blt-tokenize

# Set the entrypoint
ENTRYPOINT ["blt-tokenize"]
