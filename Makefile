export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/nettuscheduler

setup: _setup_db
	@echo -e '\e[1;31mNot Yet Implemented\e[0m'

test: _setup_db
	@cd scheduler && cargo test --all

check: _setup_db
	@cd scheduler && cargo +nightly fmt
	@cd scheduler && cargo clippy --verbose
	@cd scheduler && cargo +nightly udeps --all-targets
	@cd scheduler && cargo outdated -wR
	@cd scheduler && cargo update --dry-run

check_nightly:
	@cd scheduler && cargo +nightly clippy

install_all_prerequisite:
	@cargo install sqlx-cli --no-default-features --features postgres || true
	@cargo install cargo-outdated || true
	@cargo install cargo-udeps cargo-outdated || true

_setup_db:
	@docker-compose -f scheduler/integrations/docker-compose.yml up -d
	@cd scheduler/crates/infra && sqlx migrate run