default:
  just --list

# run commands in a Rust Docker environment
docker +args:
  #!/usr/bin/env sh
  DOCKER_BUILDKIT=1 COMPOSE_DOCKER_CLI_BUILD=1 docker compose -f dev.docker-compose.yaml run --build --rm source {{args}}
  exit_code=$?
  docker compose -f dev.docker-compose.yaml down
  exit $exit_code

# run Cargo commands in a Rust Docker environment
docker-cargo +args:
  just docker cargo --offline {{args}}

ci-build:
  just docker-cargo build

ci-test:
  just docker-cargo test

ci-bench:
  just docker-cargo bench
