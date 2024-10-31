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

build-docs:
  #!/usr/bin/env bash
  cd specification
  BUILD_OUTPUT=$(mdbook build 2>&1)

  if echo "$BUILD_OUTPUT" | grep -q '\[ERROR\]'; then
    echo "Build failed with errors:"
    echo "$BUILD_OUTPUT"
    exit 1
  else
    echo "Build completed successfully."
  fi

# Starts the ndc-spec documentation webserver
start-docs:
  cd specification && mdbook serve
