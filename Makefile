.PHONY: fmt
fmt:
	cargo fmt
	buf format -w

.PHONY: lint
lint:
	cargo clippy
	buf lint

.PHONY: test
test:
	cargo test
