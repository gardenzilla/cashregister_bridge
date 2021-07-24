.PHONY: release, test, dev, build_scanner, build_rust

test:
	cargo test

dev:
	cargo run

release:
	rm -rf bin
	mkdir bin
	cargo build --release
	cp target/release/printer_bridge bin/
	strip bin/printer_bridge