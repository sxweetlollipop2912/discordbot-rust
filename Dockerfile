FROM rust:latest as build

# create a new empty shell project
RUN USER=root cargo new --bin discordbot-rust
WORKDIR /discordbot-rust

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/discordbot_rust*
RUN cargo build --release



# our final base
FROM debian:buster-slim

# install ffmpeg
RUN apt-get update
RUN apt-get install -y ffmpeg
RUN apt-get clean

# copy the build artifact from the build stage
COPY --from=build /discordbot-rust/target/release/discordbot-rust ./

# copy necessary files
COPY ./youtube-dlc.exe ./

# set the startup command to run your binary
CMD ["./discordbot-rust"]