# ── Stage 1: build s57style Python wheel ──────────────────────────────────────
# python:3.14-slim matches the runtime image's Python minor version (3.14.x),
# so the compiled wheel targets the correct ABI without needing the full
# runtime base image in the build stage.
FROM python:3.14-slim AS builder

RUN apt-get update -qq && \
    apt-get install -y --no-install-recommends \
        curl build-essential patchelf libssl-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

# Install Rust stable toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --default-toolchain stable --profile minimal

ENV PATH="/root/.cargo/bin:${PATH}"

RUN pip install maturin -q

COPY Cargo.toml /build/Cargo.toml
COPY crates/    /build/crates/
COPY resources/ /build/resources/

RUN cd /build && \
    maturin build --release \
        --manifest-path crates/s57style-python/Cargo.toml \
        --out /wheels

# ── Stage 2: runtime image ─────────────────────────────────────────────────────
FROM ghcr.io/dirkwa/signalk-charts-provider-simple/charts-toolbox:1.0.0

# Install the pre-built wheel (no Rust toolchain in final image)
COPY --from=builder /wheels/*.whl /tmp/
RUN pip install /tmp/*.whl --break-system-packages && rm /tmp/*.whl

COPY convert.py /usr/local/bin/convert.py

WORKDIR /data
ENTRYPOINT ["python3", "/usr/local/bin/convert.py"]
