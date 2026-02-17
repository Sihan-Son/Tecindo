# Stage 1: Frontend - Build React app
FROM node:22-slim AS frontend
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm install
COPY frontend/ .
RUN npm run build

# Stage 2: Chef - Prepare cargo-chef
FROM rust:1.88-slim-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Stage 3: Planner - Analyze dependencies
FROM chef AS planner
COPY backend/ .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 4: Builder - Build dependencies and application
FROM chef AS builder
# Install SQLite development libraries
RUN apt-get update && apt-get install -y \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies (cached layer)
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY backend/ .
RUN cargo build --release

# Stage 5: Runtime - Minimal production image
FROM debian:bookworm-slim AS runtime
# Install SQLite, pandoc, XeLaTeX, CJK support, and Korean fonts for PDF export
RUN apt-get update && apt-get install -y --no-install-recommends \
    libsqlite3-0 \
    ca-certificates \
    pandoc \
    texlive-xetex \
    texlive-latex-recommended \
    texlive-fonts-recommended \
    texlive-lang-chinese \
    lmodern \
    fonts-noto-cjk \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# Create data directory
RUN mkdir -p /app/data

# Copy binary from builder
COPY --from=builder /app/target/release/tecindo /app/tecindo

# Copy frontend build
COPY --from=frontend /app/frontend/dist /app/frontend/dist

# CJK font for PDF export (override with TECINDO_CJK_FONT env var)
ENV TECINDO_CJK_FONT="Noto Sans CJK KR"

# Expose port
EXPOSE 3000

# Run the application
CMD ["/app/tecindo"]
