# syntax=docker/dockerfile:experimental
FROM rust as builder
LABEL maintainer "Automata Team"

ARG PROFILE=release
ARG TOOLCHAIN=nightly-2020-10-25

RUN apt-get update && \
	apt-get install -y --no-install-recommends cmake clang

RUN rustup toolchain install ${TOOLCHAIN} && \
    rustup default ${TOOLCHAIN} && \
    rustup target add wasm32-unknown-unknown --toolchain ${TOOLCHAIN}

WORKDIR /automata

COPY . /automata

RUN --mount=type=cache,target=/automata/target/ \
	--mount=type=cache,target=/usr/local/cargo/registry/index \
	--mount=type=cache,target=/usr/local/cargo/registry/cache \
	--mount=type=cache,target=/usr/local/cargo/git/db \
	cargo build --$PROFILE --bin automata && \
	cp /automata/target/${PROFILE}/automata /usr/local/bin/automata

# ===== SECOND STAGE ======

FROM debian:buster-slim as app
LABEL maintainer "Automata Team"

COPY --from=builder /usr/local/bin/automata /usr/local/bin/automata

RUN	useradd -m -u 1000 -U -s /bin/sh -d /automata automata && \
	mkdir -p /automata/.local/share/automata && \
	chown -R automata:automata /automata/.local && \
	ln -s /automata/.local/share/automata /data

USER automata
EXPOSE 30333 9933 9944
VOLUME ["/data"]

ENTRYPOINT ["/usr/local/bin/automata"]