# Lists all targets
default:
  just --list

lint:
  cargo clippy --all-targets --no-deps

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
