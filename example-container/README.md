# unit-rs container application example

This is a stand-alone example of a simple Hello World app built and executed
with a two-stage Containerfile.

The container image can be built with any Containerfile/Dockerfile compatible
build systems.

Example for Docker:

```sh
docker build -t rust-app
docker run -it --rm -p 8080:8080 rust-app
```

Example for Buildah/Podman:

```sh
buildah bud -t rust-app
podman run -it --rm -p 8080:8080 rust-app
```
