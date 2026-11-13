.PHONY: fmt clippy test audit deny ci

clippy:
	cargo clippy --all-targets --all
test:
	cargo test --all
audit:
	cargo audit
deny:
	cargo deny check --hide-inclusion-graph

ci: clippy test audit deny
