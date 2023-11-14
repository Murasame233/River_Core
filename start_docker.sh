docker-compose down

docker buildx build .

cargo build --target x86_64-unknown-linux-musl

cp target/x86_64-unknown-linux-musl/debug/river_core ./node1/
cp target/x86_64-unknown-linux-musl/debug/river_core ./node2/
cp target/x86_64-unknown-linux-musl/debug/river_core ./node3/

docker-compose up --force-recreate --build -d --remove-orphans