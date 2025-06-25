FROM rust:1.87 AS chef

ARG NO_CHEF=false
ENV NO_CHEF=${NO_CHEF}

RUN $NO_CHEF || cargo install cargo-chef

WORKDIR /app

FROM chef AS planner
COPY . .
RUN ($NO_CHEF && touch recipe.json) || cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN $NO_CHEF || cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .

ENV SQLX_OFFLINE=true
RUN cargo build --release --bin uc

# our final base
FROM gcr.io/distroless/cc-debian12:nonroot AS base


FROM busybox:1.37.0 AS cleaner
# small diversion through busybox to remove some files
# (no rm in distroless)

COPY --from=base / /clean

# RUN rm -r /clean/usr/lib/*-linux-gnu/libgomp*  \
#   /clean/usr/lib/*-linux-gnu/libssl*  \
#   /clean/usr/lib/*-linux-gnu/libstdc++* \
#   /clean/usr/lib/*-linux-gnu/engines-3 \
#   /clean/usr/lib/*-linux-gnu/ossl-modules \
#   /clean/usr/lib/*-linux-gnu/libcrypto.so.3 \
#   /clean/usr/lib/*-linux-gnu/gconv \
#   /clean/var/lib/dpkg/status.d/libgomp1*  \
#   /clean/var/lib/dpkg/status.d/libssl3*  \
#   /clean/var/lib/dpkg/status.d/libstdc++6* \
#   /clean/usr/share/doc/libssl3 \
#   /clean/usr/share/doc/libstdc++6 \
#   /clean/usr/share/doc/libgomp1


FROM scratch

ARG EXPIRES=Never
LABEL maintainer="robstar.pack@gmail.com" quay.expires-after=${EXPIRES}

COPY --from=cleaner /clean /

# copy the build artifact from the build stage
COPY --from=builder /app/target/release/uc /home/nonroot/uc

# # set the startup command to run your binary
ENTRYPOINT ["/home/nonroot/uc"]
