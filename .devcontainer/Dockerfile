# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.245.2/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

# Pret vhs for install
RUN sudo mkdir -p /etc/apt/keyrings
RUN curl -fsSL https://repo.charm.sh/apt/gpg.key | sudo gpg --dearmor -o /etc/apt/keyrings/charm.gpg
RUN echo "deb [signed-by=/etc/apt/keyrings/charm.gpg] https://repo.charm.sh/apt/ * *" | sudo tee /etc/apt/sources.list.d/charm.list

# Install ttyd binary
RUN sudo wget https://github.com/tsl0922/ttyd/releases/download/1.7.1/ttyd.x86_64
RUN sudo mv ./ttyd.x86_64 /usr/bin/ttyd
RUN sudo chmod +rwx /usr/bin/ttyd

# Thid is for generating the terminal gif files
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y vhs install libnss3-dev ffmpeg libatk1.0-0 libatk-bridge2.0-0 libcups2-dev libxcomposite-dev
