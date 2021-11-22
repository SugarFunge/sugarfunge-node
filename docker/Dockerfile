# Based on https://github.com/paritytech/polkadot/blob/master/scripts/dockerfiles/polkadot/polkadot_builder.Dockerfile

FROM paritytech/ci-linux:production AS builder
WORKDIR /sugarfunge
COPY . /sugarfunge
RUN cargo build --locked --release

FROM ubuntu:20.04

COPY --from=builder /sugarfunge/target/release/sugarfunge-node /usr/local/bin

# Install required packages to run [--dev only]
RUN apt-get update && \
  apt-get install --no-install-recommends -y curl git gnupg ca-certificates

RUN useradd -m -u 1000 -U -s /bin/sh -d /sugarfunge-node sugarfunge-node && \
	mkdir -p /data /sugarfunge-node/.local/share && \
	chown -R sugarfunge-node:sugarfunge-node /data && \
	ln -s /data /sugarfunge-node/.local/share/sugarfunge-node && \
# unclutter and minimize the attack surface [Disabled for --dev]
#	rm -rf /usr/bin /usr/sbin && \
# check if executable works in this container
	/usr/local/bin/sugarfunge-node --version

USER sugarfunge-node

EXPOSE 30333 9615 9933 9944

ENTRYPOINT ["/usr/local/bin/sugarfunge-node"]

CMD ["--dev", "--pruning", "archive", "--tmp"]

# CMD ["--dev", "--pruning", "archive", "--tmp", "--ws-external", "--rpc-external", "--rpc-cors", "all"]