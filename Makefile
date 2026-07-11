.PHONY: test release

test:
	cargo fmt --check
	cargo clippy --all-targets --all-features -- -D warnings
	cargo test --all-targets --all-features --locked -- --test-threads=1
	node scripts/check-lessons.js
	npm run test:launcher
	@home=$$(mktemp -d); trap 'rm -rf "$$home"' EXIT; PRACTICODE_HOME="$$home" PRACTICODE_NO_UPDATE_CHECK=1 cargo run --locked -- --smoke
	npm pack --dry-run

release:
	@scripts/release.sh $(VERSION)
