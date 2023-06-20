FROM rustlang/rust:nightly AS builder

RUN rustup target add wasm32-unknown-unknown

RUN apt-get update && apt-get install -y --no-install-recommends \
        clang \
        libclang-dev \
        protobuf-compiler \
        cmake

WORKDIR /sugarfunge

COPY . /sugarfunge

RUN cargo build --locked --release

FROM gcr.io/distroless/cc-debian11

COPY --from=builder /lib/x86_64-linux-gnu/libz.so.1 /lib/x86_64-linux-gnu/libz.so.1

COPY --from=builder /sugarfunge/target/release/sugarfunge-node /

ENTRYPOINT ["/sugarfunge-node"]

CMD ["--dev", "--pruning", "archive", "--tmp"]
