.PHONY:main clean run
RUST_LOG?=error
main:
	cargo build && RUST_LOG=${RUST_LOG} RUST_BACKTRACE=1 ./target/debug/json2model ./res/sample.json

clean:
	cargo clean
test:
	cargo build && RUST_LOG=${RUST_LOG} RUST_BACKTRACE=2 ./target/debug/json2model ./res/test.json


