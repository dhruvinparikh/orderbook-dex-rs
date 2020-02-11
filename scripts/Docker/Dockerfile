FROM debian:stretch-slim

# metadata
ARG VCS_REF
ARG BUILD_DATE

# TODO: Add some labels for future reference
# show backtraces
ENV RUST_BACKTRACE 1

# install tools and dependencies
RUN apt-get update && \
	DEBIAN_FRONTEND=noninteractive apt-get upgrade -y && \
	DEBIAN_FRONTEND=noninteractive apt-get install -y \
		libssl1.1 \
		ca-certificates \
		curl && \
# apt cleanup
	apt-get autoremove -y && \
	apt-get clean && \
	find /var/lib/apt/lists/ -type f -not -name lock -delete; \
# add user
	useradd -m -u 1000 -U -s /bin/sh -d /dnachain dnachain

# add Metaverse DNA binary to docker image
COPY dnachain /usr/local/bin/
RUN chmod 777 /usr/local/bin/dnachain

USER dnachain

# check if executable works in this container
# RUN which dnachain
RUN /usr/local/bin/dnachain --version

EXPOSE 30333 9933 9944
VOLUME ["/dnachain"]

ENTRYPOINT ["/usr/local/bin/dnachain"]
