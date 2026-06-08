# 🐳 Docker Deployment

> [!WARNING]
> **Docker introduces I/O and networking overhead.** Running a nanosecond-scale database like CLUAIZD inside a container will slow down LMDB memory mapping and weaken the engine's superpower. 
> 
> For maximum performance on laptops, servers, or edge devices, we highly recommend **[Bare-Metal Native Deployment](file:///c:/Users/Aryan/my/Cluaiz-workspace/Cluaiz-Technologies/cluaizd/docs/deployment/bare-metal.md)** instead.

While CLUAIZD can run in Docker, it is primarily recommended for staging, testing, or environments where ease of deployment outweighs raw performance.

## The Dockerfile

Create a `Dockerfile` in the root of the project:

```dockerfile
# Build Stage
FROM rust:1.80-slim AS builder
WORKDIR /usr/src/cluaizd

# Install build dependencies (CMake required for LMDB)
RUN apt-get update && apt-get install -y cmake g++ clang

# Copy source
COPY . .

# Build release binary
RUN cargo build --release -p cluaizd-server

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /usr/src/cluaizd/target/release/cluaizd-server /usr/local/bin/cluaizd

# Create data directory
RUN mkdir -p /var/lib/cluaizd
ENV CLUAIZD_DATA_DIR=/var/lib/cluaizd

# Expose the API and WebSocket port
EXPOSE 7331

# Run the server
CMD ["cluaizd"]
```

## Running with Docker Compose

For production deployments, we highly recommend using Docker Compose with volume mounts for the data directory.

`docker-compose.yml`:
```yaml
version: '3.8'

services:
  cluaizd:
    build: .
    ports:
      - "7331:7331"
    volumes:
      - cluaizd_data:/var/lib/cluaizd
    environment:
      - RUST_LOG=info,cluaizd=debug
    deploy:
      resources:
        limits:
          memory: 16G # Recommend hard limits so the Dreamer GC works effectively

volumes:
  cluaizd_data:
```

Start the server:
```bash
docker-compose up -d
```

## Performance Note
> [!WARNING]  
> If you are running Docker Desktop on macOS or Windows, disk I/O is heavily virtualized. This will severely impact LMDB write speeds. For performance testing, run Docker natively on Linux or use a bare-metal Linux machine.
