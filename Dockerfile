FROM centos:centos8

RUN dnf update -y && \
  dnf install -y \
  gcc \
  git \
  openssl-devel \
  sqlite \
  sqlite-devel

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > script.sh \
  && bash script.sh -y

RUN mkdir project

WORKDIR project

COPY /Cargo.lock ./Cargo.lock
COPY /Cargo.toml ./Cargo.toml
COPY /src ./src

RUN /root/.cargo/bin/cargo build --release && \
  strip ./target/release/typeracer && \
  cp ./target/release/typeracer ./typeracer && \
  /root/.cargo/bin/cargo clean

ENTRYPOINT ["sleep", "infinity"]

