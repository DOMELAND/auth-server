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

## Test    
 To test the DOMELAND Account web sevice, following are some cases ( tools: postman):

##### API: ping-pong Test
```
 URL:  http://localhost:19253/ping
 Method: GET
```

##### API: account register 
```
 URL:  http://localhost:19253/register
 Method: POST
 Body (Json):
 {
   "username":"max123",
   "password":"123456",
   "ethaddr":"0x9c5Eb6CcB92e551ec1671cdafF7b55d44A28615b"
 } 
 ```
 
##### API: generate one-time access token to  game-server
```
 URL: http://localhost:19253/generate_token
 Method: POST
 Body (Json):
 {
   "username":"max123",
   "password":"123456"
 } 
 ```

##### API: verify one-time token
```
 URL: http://localhost:19253/verify
 Method: POST
 Body (Json):
 {
    "token": {
        "unique": 15183996567503823849
    }
 }
```
##### API: query username by uuid
```
URL: http://localhost:19253/uuid_to_username
Method: POST
Body (Json):
{
    "uuid": "6cfc2a33-5ea9-456b-bfdf-4c88e7b99bd4"
}
```

##### API: query uuid by username
```
URL: http://localhost:19253/username_to_uuid
Method: POST
Body (Json):
{
    "username": "max123"
}
```

##### API: query userinfo by ethereum address
```
URL: http://localhost:19253/eth_to_info
Method: POST
Body (Json):
{
    "ethaddr": "0x8c5Eb6CcB92e551ec1671cdafF7b55d44A28615a"
}
```
##### API: query userinfo by username
```
URL: http://localhost:19253/username_to_info
Method: POST
Body (Json):
{
    "username": "max"
}
```
##### API: query userinfo by uuid
```
URL: http://localhost:19253/uuid_to_info
Method: POST
Body (Json):
{
    "uuid": "6cfc2a33-5ea9-456b-bfdf-4c88e7b99bd4"
}
```