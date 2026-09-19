# #############################################################################
# MARK: Plugin build

# Build and test the plugin independently of the proto version under test.
FROM docker.io/library/rust:1-bookworm AS plugin

WORKDIR /workspace
RUN rustup target add wasm32-wasip1

COPY Cargo.toml Cargo.lock build.rs rustfmt.toml ./
COPY src ./src

RUN cargo test --locked \
  && cargo build --locked --target wasm32-wasip1 --release

# #############################################################################
# MARK: Isolated test environment

# The runtime stage has its own proto installation and no host-mounted state.
FROM docker.io/library/debian:bookworm-slim AS test

ARG PROTO_VERSION=0.59.0

ENV PROTO_HOME=/opt/proto
ENV PATH="/opt/proto/bin:/opt/proto/shims:${PATH}"

# Include the native toolchain needed when exercising GraalVM Native Image.
RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates curl git unzip xz-utils build-essential zlib1g-dev \
  && rm -rf /var/lib/apt/lists/*

# #############################################################################
# MARK: Proto installation

# Verify the requested version so installer changes cannot silently change it.
RUN curl -fsSL https://moonrepo.dev/install/proto.sh -o /tmp/install-proto.sh \
  && bash /tmp/install-proto.sh "${PROTO_VERSION}" --yes --no-profile \
  && rm /tmp/install-proto.sh \
  && test "$(proto --version)" = "proto ${PROTO_VERSION}"

# #############################################################################
# MARK: Integration test entry point

COPY --from=plugin /workspace/target/wasm32-wasip1/release/proto_graalvm_native_image_plugin.wasm \
  /workspace/target/wasm32-wasip1/release/proto_graalvm_native_image_plugin.wasm
COPY proto_test/.prototools proto_test/test.sh /workspace/proto_test/

WORKDIR /workspace/proto_test

# Install GraalVM at runtime so each new container tests a clean installation.
CMD ["bash", "./test.sh"]
