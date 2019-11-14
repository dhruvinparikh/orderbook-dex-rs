FROM phusion/baseimage:0.11 as builder
LABEL maintainer "info@blockxlabs.com"
LABEL description="This is the build stage for Metaverse-DNA. Here we create the binary."

ARG PROFILE=release
WORKDIR /metaverse-dna

COPY . /metaverse-dna

RUN apt-get update && \
	apt-get upgrade -y && \
	apt-get install -y cmake pkg-config libssl-dev git clang
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
        export PATH=$PATH:$HOME/.cargo/bin && \
        /metaverse-dna/scripts/init.sh && \
        cargo build --$PROFILE

# ===== SECOND STAGE ======

FROM phusion/baseimage:0.11
LABEL maintainer "info@blockxlabs.com"
LABEL description="This is the 2nd stage: a very small image where we copy the Metaverse-DNA binary."
ARG PROFILE=release
COPY --from=builder /metaverse-dna/target/$PROFILE/metaverse-dna /usr/local/bin

RUN mv /usr/share/ca* /tmp && \
	rm -rf /usr/share/*  && \
	mv /tmp/ca-certificates /usr/share/ && \
	rm -rf /usr/lib/python* && \
	useradd -m -u 1000 -U -s /bin/sh -d /metaverse-dna metaverse-dna && \
	mkdir -p /metaverse-dna/.local/share/metaverse-dna && \
	chown -R metaverse-dna:metaverse-dna /metaverse-dna/.local && \
	ln -s /metaverse-dna/.local/share/metaverse-dna /data && \
	rm -rf /usr/bin /usr/sbin

USER metaverse-dna
EXPOSE 30333 9933 9944
VOLUME ["/data"]

CMD ["/usr/local/bin/metaverse-dna"]
