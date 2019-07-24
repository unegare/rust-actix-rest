FROM debian

RUN apt-get update && \
    apt-get -y upgrade && \
    apt-get -y install git curl g++ build-essential

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

WORKDIR /usr/src/app

#RUN git clone https://github.com/unegare/rust-actix-rest.git
#RUN ["/bin/bash", "-c", "source $HOME/.cargo/env; cd ./rust-actix-rest/; cargo build --release; mkdir uploaded"]

COPY . .
RUN ["/bin/bash", "-c", "source $HOME/.cargo/env; cargo build --release;"]


EXPOSE 8080

#ENTRYPOINT ["/bin/bash", "-c", "source $HOME/.cargo/env; cd ./rust-actix-rest/; cargo run --release"]
ENTRYPOINT ["/bin/bash", "-c", "source $HOME/.cargo/env; cargo run --release"]
