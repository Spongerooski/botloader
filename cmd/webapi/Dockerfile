# You need to build the base image first and tag it as botloader-base
FROM botloader-base as builder
RUN cargo build --release --bin webapi

#run
FROM debian:buster-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/webapi /usr/local/bin/botloader-webapi

EXPOSE 7447

ENTRYPOINT ["/usr/local/bin/botloader-webapi"]