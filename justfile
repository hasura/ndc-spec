default:
  just --list

# run commands in a rust docker environment
docker +args:
  DOCKER_BUILDKIT=1 COMPOSE_DOCKER_CLI_BUILD=1 docker compose -f dev.docker-compose.yaml run --build --rm source {{args}}

ci-build:
  just docker cargo build

ci-test:
  just docker cargo test

ci-bench:
  just docker cargo bench