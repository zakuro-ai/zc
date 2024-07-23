FROM zakuroai/compute

USER root
RUN sudo apt update && sudo apt install -y \
    make

RUN rustup default stable

COPY ./ /workspace
RUN make build_zc
