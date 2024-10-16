# Lists all targets
default:
  just --list

# run commands in a rust docker environment
docker +args:
  #!/usr/bin/env sh
  DOCKER_BUILDKIT=1 COMPOSE_DOCKER_CLI_BUILD=1 docker compose -f dev.docker-compose.yaml run --build --rm --env=RUSTFLAGS='-D warnings' source {{args}}
  exit_code=$?
  docker compose -f dev.docker-compose.yaml down
  exit $exit_code

# Builds in docker
ci-build:
  just docker cargo build

# Runs the tests in docker
ci-test:
  just docker cargo test

# Runs linting checks in docker
ci-lint:
  just docker cargo clippy

# Runs benchmarks in docker
ci-bench:
  just docker cargo bench

# Runs the tests
test *ARGS:
  #!/usr/bin/env bash
  if command -v cargo-nextest; then
    COMMAND=(cargo nextest run)
  else
    COMMAND=(cargo test)
  fi
  COMMAND+=(--no-fail-fast "$@")
  echo "${COMMAND[*]}"
  "${COMMAND[@]}"

# Formats all the Markdown, Rust, Nix etc
fix-format: fix-format-prettier
  cargo fmt --all
  ! command -v nix || nix fmt

# Formats Markdown, etc with prettier
fix-format-prettier:
  npx --yes prettier --write .

# Runs the tests and updates all goldenfiles with the test output
update-golden-files:
  UPDATE_GOLDENFILES=1 just test
  just fix-format-prettier

# Starts the ndc-spec documentation webserver
start-docs:
  cd specification && mdbook serve
