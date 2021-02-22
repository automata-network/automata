FROM rust as builder
LABEL maintainer "Automata Team"

ARG PROFILE=release
ARG TOOLCHAIN=nightly-2020-10-25

RUN apt-get update && \
	apt-get install -y --no-install-recommends cmake clang

WORKDIR /automata

COPY . /automata

RUN rustup toolchain install ${TOOLCHAIN} && \
    rustup default ${TOOLCHAIN} && \
    rustup target add wasm32-unknown-unknown --toolchain ${TOOLCHAIN}

RUN cargo build --$PROFILE --bin automata

# ===== SECOND STAGE ======

FROM debian:buster-slim as app
LABEL maintainer "Automata Team"

ARG PROFILE=release
COPY --from=builder /automata/target/$PROFILE/automata /usr/local/bin

RUN	useradd -m -u 1000 -U -s /bin/sh -d /automata automata && \
	mkdir -p /automata/.local/share/automata && \
	chown -R automata:automata /automata/.local && \
	ln -s /automata/.local/share/automata /data

USER automata
EXPOSE 30333 9933 9944
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/automata"]