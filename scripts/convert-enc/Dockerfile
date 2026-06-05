# ── Stage 1: build and pre-install the s57style Python wheel ──────────────────
# python:3.14-slim matches the runtime image's Python minor version (3.14.x).
# We install the wheel into /install so it can be copied without pip at runtime.
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

# Build wheel, then install to /install so the runtime stage can COPY it in
RUN cd /build && \
    PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 \
    maturin build --release \
        --manifest-path crates/s57style-python/Cargo.toml \
        --out /wheels && \
    pip install --prefix=/install /wheels/*.whl

# ── Stage 2: runtime image ─────────────────────────────────────────────────────
FROM ghcr.io/dirkwa/signalk-charts-provider-simple/charts-toolbox:1.0.0

# pip --prefix installs to site-packages; the runtime Python (Debian-based)
# resolves imports from dist-packages, so copy there explicitly.
COPY --from=builder /install/lib/python3.14/site-packages/ /usr/local/lib/python3.14/dist-packages/

COPY convert.py /usr/local/bin/convert.py

WORKDIR /data
ENTRYPOINT ["python3", "/usr/local/bin/convert.py"]
