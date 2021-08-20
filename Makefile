setup:
	@echo -e '\e[1;31mNot Yet Implemented\e[0m'

test: 
	@docker-compose -f scheduler/integrations/docker-compose.yml up -d
	@cd scheduler && export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/nettuscheduler && cargo test --all

check:
	@cd scheduler && cargo +nightly fmt
	@cd scheduler && cargo clippy --verbose
	@cd scheduler && cargo +nightly udeps --all-targets
	@cd scheduler && cargo outdated -wR
	@cd scheduler && cargo update --dry-run

check_nightly:
	@cd scheduler && cargo +nightly clippy
