FROM rust
COPY Carlito /root/Carlito/
WORKDIR /root/Carlito/source
RUN cargo update && cargo build
WORKDIR /root/Carlito
ENTRYPOINT [ "./debug.sh" ]