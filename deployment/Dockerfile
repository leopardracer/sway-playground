# Stage 1: Build
FROM lukemathwalker/cargo-chef:latest-rust-1.82 as chef
WORKDIR /build/
# hadolint ignore=DL3008

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    lld \
    clang \
    libclang-dev \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Build sway-playground
FROM chef as planner
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder

# Install charcoal
RUN cargo install --git https://github.com/FuelLabs/charcoal.git --rev e69a6ffaf3e7eaf9f3ceea543087ea59ec5fd5d1

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
COPY --from=planner /build/recipe.json recipe.json
# Build our project dependecies, not our application!
RUN cargo chef cook --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
RUN cargo build

# Stage 2: Run
FROM ubuntu:22.04 as run

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates curl git pkg-config libssl-dev \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /root/

COPY --from=builder /build/target/debug/sway-playground .
COPY --from=builder /build/target/debug/sway-playground.d .
COPY --from=builder /build/Rocket.toml .
COPY --from=builder /build/projects projects
COPY --from=builder /usr/local/cargo/bin/charcoal /bin

# Install fuelup
RUN curl -fsSL https://install.fuel.network/ | sh -s -- --no-modify-path
ENV PATH="/root/.fuelup/bin:$PATH"

# Install all fuel toolchains
RUN fuelup toolchain install latest
RUN fuelup toolchain install nightly
RUN fuelup toolchain install testnet
RUN fuelup toolchain install mainnet

# Install the forc dependencies
RUN fuelup default testnet
RUN forc build --path projects/swaypad

EXPOSE 8080

CMD ["./sway-playground"]
