FROM debian:10
RUN apt-get update && apt-get upgrade -y
RUN apt-get install -y gcc gcc-multilib make musl-tools curl
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN /bin/bash -c "source $HOME/.cargo/env && rustup target add x86_64-unknown-linux-musl"
COPY . /opt/appbuild
WORKDIR /opt/appbuild/server
RUN /bin/bash -c "source $HOME/.cargo/env && cargo build --release --target x86_64-unknown-linux-musl"

FROM scratch
WORKDIR /opt/app
COPY --from=0 /opt/appbuild/target/x86_64-unknown-linux-musl/release/auth-server .
EXPOSE 19253
CMD ["./auth-server"]
