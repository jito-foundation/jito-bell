# Makefile for Jito Bell

.PHONY: lint
lint:
	cargo sort --workspace --check
	cargo fmt --all --check
	cargo clippy

.PHONY: test
test:
	cargo test

# Build release
.PHONY: build-release
build-release:
	cargo build --release

# Docker Compose up
.PHONY: docker-compose-up
docker-compose-up:
	docker compose --env-file .env up -d

# Docker Compose down
.PHONY: docker-compose-down
docker-compose-down:
	docker compose down

# Docker Compose stop
.PHONY: docker-stop
docker-stop:
	docker stop jito-bell

# Docker Compose Rebuild 
.PHONY: docker-rebuild
docker-rebuild:
	docker rm jito-bell; docker rmi jito-bell; docker compose --env-file .env up -d --build

