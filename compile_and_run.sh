#!/bin/sh

set -e # Exit early if any commands fail

# Copied from .codecrafters/compile.sh
#
# - Edit this to change how your program compiles locally
# - Edit .codecrafters/compile.sh to change how your program compiles remotely
(
  cd "$(dirname "$0")" # Ensure compile steps are run within the repository directory
  cargo build --release --target-dir=/tmp/codecrafters-build-http-server-rust --manifest-path Cargo.toml
)

# Copied from .codecrafters/run.sh
#
# - Edit this to change how your program runs locally
# - Edit .codecrafters/run.sh to change how your program runs remotely
exec /tmp/codecrafters-build-http-server-rust/release/flyweight-http-server "$@"
