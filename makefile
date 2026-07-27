build:
	cargo build

full-build:
	cargo build --all-features

test:
	cargo test

full-test:
	cargo test --all-features

lint:
	cargo clippy --all-features --all-targets -- -Dwarnings

format:
	cargo fmt
