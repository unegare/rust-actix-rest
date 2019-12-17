FROM scratch as rsync

WORKDIR /app

COPY . .


FROM debian:latest as builder

RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install ca-certificates curl gcc build-essential && \
    apt-get clean

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

WORKDIR /usr/src/app

COPY ./Cargo.toml ./

RUN mkdir src && echo "fn main() {}" > ./src/main.rs

RUN bash -c 'source $HOME/.cargo/env && cargo build --release'

RUN rm ./target/release/deps/actix_003* && rm ./target/release/actix_003

COPY --from=rsync /app ./rsync

RUN sh -c 'rm -f ./rsync/Cargo.{toml,lock} && rm -rf ./src && mv ./rsync/* ./ && rm -rf ./rsync'

RUN bash -c 'source $HOME/.cargo/env && cargo build --release'

RUN strip ./target/release/actix_003


FROM debian:latest as runner

RUN apt-get update && apt-get install -y ca-certificates && apt-get clean

RUN update-ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/actix_003 .

CMD ./actix_003

EXPOSE 8080
