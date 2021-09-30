# syntax=docker/dockerfile:experimental
FROM rust as builder
LABEL maintainer "Automata Team"

ARG PROFILE=release
ARG TOOLCHAIN=nightly-2021-06-16

RUN apt-get update && \
	apt-get install -y --no-install-recommends cmake clang curl

RUN rustup toolchain install ${TOOLCHAIN} && \
    rustup default ${TOOLCHAIN} && \
    rustup target add wasm32-unknown-unknown --toolchain ${TOOLCHAIN}

ARG SCCACHE_TAR_URL=https://github.com/mozilla/sccache/releases/download/v0.2.15/sccache-v0.2.15-x86_64-unknown-linux-musl.tar.gz

RUN curl -LsSf ${SCCACHE_TAR_URL} > /tmp/sccache.tar.gz && \
	tar axvf /tmp/sccache.tar.gz --strip-components=1 -C /usr/local/bin --wildcards --no-anchored 'sccache' && \
	chmod +x /usr/local/bin/sccache && \
	sccache --version && \
	rm -rf /tmp/sccache.tar.gz

ENV RUSTC_WRAPPER=/usr/local/bin/sccache
ENV CARGO_INCREMENTAL=0
ENV SCCACHE_CACHE_SIZE=1G

WORKDIR /automata

COPY . /automata

ARG FEATURES

RUN --mount=type=cache,target=/root/.cache/sccache \
	--mount=type=cache,target=/usr/local/cargo/registry/index \
	--mount=type=cache,target=/usr/local/cargo/registry/cache \
	--mount=type=cache,target=/usr/local/cargo/git/db \
	cargo build --$PROFILE --bin automata --features $FEATURES && \
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
