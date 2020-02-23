FROM rust:1.40

WORKDIR /opt/hlwall
RUN cargo install cargo-do
COPY . /opt/hlwall
RUN cargo install --path .

ENV RUST_LOG=hlwall=trace
CMD ["hlwall", "-l", "0.0.0.0:27016", "hlds:27015"]
