.PHONY:main clean run
RUST_LOG?=debug
main:
	cargo build && RUST_LOG=${RUST_LOG} ./target/debug/json2model ./res/sample.json

clean:
	cargo clean

