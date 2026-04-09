tree:
	cargo tree

clean:
	cargo clean

build-debug:
	cargo build

build-release:
	cargo build --release

arp: build-debug
	RUST_LOG=info,netprobe=debug \
	sudo ./target/debug/netprobe arp --ip 192.168.50.49

ndp: build-debug
	RUST_LOG=info,netprobe=debug \
	sudo ./target/debug/netprobe ndp --ip fe80::9209:d0ff:fe32:396f
