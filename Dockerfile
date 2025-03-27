FROM mirror.gcr.io/rust:1.85 AS builder
WORKDIR /build
COPY . /build
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /build/target/release/llm-natsuki-bot /
CMD [ "/llm-natsuki-bot" ]
