tree:
	cargo tree

clean:
	cargo clean

build-debug:
	cargo build

build-release:
	cargo build --release

run-debug: build-debug
	RUST_LOG=info,netprobe=debug \
	sudo ./target/debug/netprobe arp --ip 192.168.50.49

run-release: build-release
	./target/release/netprobe