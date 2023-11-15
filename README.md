Due to a deadline, this repo only contains necessary code. There are some features still in developing, for a better perception of demo, unfinished code is delete in this repo.

# River

River is a real time layer-2 network based on Algorand.

Written in Rust.
# Ports
Current Port definition

- `2999` API Port
- `3000` DATA Port
- `3001` PEER Port

# How to run the example
### Pre Require

- docker-compose
- Rust for compile

if you want to run in the docker, the target must be `x86_64-unknown-linux-musl`

## RunNode
#### Run Through docker
```
bash start_docker.sh
```
This file will start three node. want to know more, check [docker-compose.yml](docker-compose.yml)
#### Run Locally
```
cargo run --package river_core --bin river_core
```
## Run example client
```
cargo run --package river_core --bin client_1
```
you can check the file [client_1.rs](src/bin/client_1.rs) for more information

## Contact
```
me@murasame.moe
```
