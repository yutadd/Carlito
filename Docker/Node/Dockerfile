FROM rust
COPY ../../ /root/Carlito/
WORKDIR /root/Carlito/source
RUN cargo update
WORKDIR /root/Carlito
ENTRYPOINT [ "./debug.sh" ]