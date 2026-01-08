# Divine AGI V16 — Pre-built Binary Deployment
# Just copy the locally compiled binary - NO compilation on Railway!

FROM debian:bookworm-slim

WORKDIR /app

# Install only runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy pre-built binary (compiled locally)
COPY divine-agi /app/divine-agi

# Make executable
RUN chmod +x /app/divine-agi

# Expose port
EXPOSE 8080

# Environment
ENV PORT=8080
ENV RUST_LOG=info

# Run
CMD ["sh", "-c", "./divine-agi server --port ${PORT:-8080}"]
