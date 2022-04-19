# auth

This is the implementation of the auth server and command-line client for Domeland/Veloren.

## Dependencies

The Auth server is implemented using Rust.
For more information about Veloren development, please refer to: https://book.veloren.net/

## Build the server
To build the server, you can simply run the following command: `cargo build`

## Setting up your own auth server

### Local server
You can run a local server with the following command: `cargo run`.

### Docker image
For a deployment-ready server, you can build docker image using `./build-server-dockerimage.sh` or without cloning the repo `docker build -t auth-server:latest https://gitlab.com/veloren/auth.git`. Docker will have to be installed.

### Run the auth server as a service using Docker Compose
A docker compose file is also provided to provide the auth server as a service. You'll need docker-compose for that.

#### Deployment notice
 To keep your data secured, it is essential to setup the server to be connected to through a public network run behind a TLS terminator such as nginx
