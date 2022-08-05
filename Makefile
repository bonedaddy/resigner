.PHONY: build
build:
	(cargo build && cp target/debug/cli resigner)

.PHONY: lint
lint:
	cargo +nightly clippy --fix -Z unstable-options --release --all --allow-dirty

.PHONY: fmt
fmt:
	find -type f -name "*.rs" -not -path "*target*" -exec rustfmt --edition 2021 {} \;