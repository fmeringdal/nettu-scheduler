setup:
	@echo -e '\e[1;31mNot Yet Implemented\e[0m'

test:
	@echo -e '\e[1;31mNot Yet Implemented\e[0m'

check:
	@cd scheduler && cargo +nightly fmt
	@cd scheduler && cargo clippy --verbose
	@cd scheduler && cargo +nightly udeps --all-targets
	@cd scheduler && cargo outdated -wR
	@cd scheduler && cargo update --dry-run

check_nightly:
	@cd scheduler && cargo +nightly clippy
