# A container that builds the application using Rust
FROM docker.io/rust:slim-bullseye AS builder

# Install curl for the next command, and libclang-dev for Rust's bindgen to work
RUN apt-get -y update && apt-get -y install curl libclang-dev

# Add the repository using the instructions at https://unit.nginx.org/installation/
RUN curl --output /usr/share/keyrings/nginx-keyring.gpg  \
    https://unit.nginx.org/keys/nginx-keyring.gpg

RUN echo "deb [signed-by=/usr/share/keyrings/nginx-keyring.gpg] https://packages.nginx.org/unit/debian/ bullseye unit" > /etc/apt/sources.list.d/unit.list && \
    echo "deb-src [signed-by=/usr/share/keyrings/nginx-keyring.gpg] https://packages.nginx.org/unit/debian/ bullseye unit" >> /etc/apt/sources.list.d/unit.list

RUN apt-get -y update

# Note: the version of unit-dev here must match the version of the nginx/unit
# image below.
# By the time you are reading this, the version here (and of the runtime image
# below) will most likely be out of date, and should be updated.
RUN apt-get -y install unit=1.27.0-1~bullseye unit-dev=1.27.0-1~bullseye

WORKDIR /usr/src/unit-rs-app
COPY . .

RUN cargo install --path .

RUN which unit-rs-app


# A container that runs the application with NGINX Unit
FROM docker.io/nginx/unit:1.27.0-minimal

# Copy the binary from the builder image above; libunit.a is statically linked
# into it, and the resulting binary works with any NGINX Unit server with a
# matching version number (including images for other languages).
COPY --from=builder /usr/local/cargo/bin/unit-rs-app /

# Configure the app on port 8080 with a json file
COPY app-config.json /docker-entrypoint.d/

EXPOSE 8080/tcp
