# Stage 1: Chef - Prepare cargo-chef
FROM rust:1.84-slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Stage 2: Planner - Analyze dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - Build dependencies and application
FROM chef AS builder
# Install SQLite development libraries
RUN apt-get update && apt-get install -y \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies (cached layer)
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release

# Stage 4: Runtime - Minimal production image
FROM debian:bookworm-slim AS runtime
# Install SQLite runtime libraries
RUN apt-get update && apt-get install -y \
    libsqlite3-0 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# Create data directory
RUN mkdir -p /app/data

# Copy binary from builder
COPY --from=builder /app/target/release/tecindo /app/tecindo

# Expose port
EXPOSE 3000

# Run the application
CMD ["/app/tecindo"]
