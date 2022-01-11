# Based on https://github.com/paritytech/polkadot/blob/master/scripts/dockerfiles/polkadot/polkadot_builder.Dockerfile

FROM paritytech/ci-linux:production AS builder
WORKDIR /sugarfunge
COPY . /sugarfunge
RUN cargo build --locked --release

FROM gcr.io/distroless/cc-debian11

COPY --from=builder /sugarfunge/target/release/sugarfunge-node /

ENTRYPOINT ["/sugarfunge-node"]

CMD ["--dev", "--pruning", "archive", "--tmp"]

# CMD ["--dev", "--pruning", "archive", "--tmp", "--ws-external", "--rpc-external", "--rpc-cors", "all"]
