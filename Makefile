all: fmt lint

fmt:
	cargo +nightly fmt

lint:
	cargo clippy
	# cargo clippy --tests

update:
	cargo update
