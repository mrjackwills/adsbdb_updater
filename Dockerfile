#############
## Builder ##
#############

FROM rust:slim as BUILDER

WORKDIR /usr/src

# Create blank project
RUN cargo new adsbdb_updater

# We want dependencies cached, so copy those first
COPY Cargo.* /usr/src/adsbdb_updater/

# Set the working directory
WORKDIR /usr/src/adsbdb_updater

# This is a dummy build to get the dependencies cached - probably not needed - as run via a github action
RUN cargo build --release

# Now copy in the rest of the sources
COPY src /usr/src/adsbdb_updater/src/

## Touch main.rs to prevent cached release build
RUN touch /usr/src/adsbdb_updater/src/main.rs

# This is the actual application build
RUN cargo build --release

#############
## Runtime ##
#############

# Could actually use a scratch container - just need to build for musl!

FROM debian:bullseye-slim AS RUNTIME

ARG DOCKER_GUID=1000 \
	DOCKER_UID=1000 \
	DOCKER_TIME_CONT=Europe\
	DOCKER_TIME_CITY=Berlin \
	DOCKER_APP_USER=app_user \
	DOCKER_APP_GROUP=app_group

ENV TZ=${DOCKER_TIME_CONT}/${DOCKER_TIME_CITY}

RUN apt-get update \
	&& groupadd --gid ${DOCKER_GUID} ${DOCKER_APP_GROUP} \
	&& useradd --no-create-home --no-log-init --uid ${DOCKER_UID} --gid ${DOCKER_GUID} ${DOCKER_APP_USER}

WORKDIR /app

COPY --from=BUILDER /usr/src/adsbdb_updater/target/release/adsbdb_updater /app/

# Use an unprivileged user
USER ${DOCKER_APP_USER}

CMD ["/app/adsbdb"]