FROM rustembedded/cross:aarch64-unknown-linux-gnu-0.2.1

RUN apt update
RUN apt install -y gcc-aarch64-linux-gnu

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain stable --target aarch64-unknown-linux-gnu -y
RUN source $HOME/.cargo/env
