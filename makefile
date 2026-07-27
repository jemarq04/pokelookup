build:
	cargo build

build-full:
	cargo build --all-features

clean:
	cargo clean

test:
	cargo test

test-full:
	cargo test --all-features

lint:
	cargo clippy --all-features --all-targets -- -Dwarnings

format:
	cargo fmt
