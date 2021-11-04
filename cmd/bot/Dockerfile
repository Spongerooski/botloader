# You need to build the base image first and tag it as botloader-base
FROM botloader-base as builder
RUN cargo build --release --bin bot

#run
FROM debian:buster-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/bot /usr/local/bin/botloader-bot
ENTRYPOINT ["/usr/local/bin/botloader-bot"]