#!/usr/bin/env bash

# #############################################################################
# MARK: Test environment

set -euo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
export PROTO_CONFIG_MODE=local

# #############################################################################
# MARK: Installation and execution

# Install from the fixture configuration before checking the command entry point.
proto --version

if ! proto use; then
  # Keep diagnostics visible even when the container is removed with --rm.
  for log in ./*-install.log; do
    if [[ -f "$log" ]]; then
      cat "$log" >&2
    fi
  done

  exit 1
fi

proto run graalvm-native-image -- --version

# #############################################################################
# MARK: Shell activation

# Check activation as well as execution through proto.
original_java_home=${JAVA_HOME-}
activation=$(proto activate bash --export)
eval "$activation"

test -n "${GRAALVM_HOME:-}"
test -d "$GRAALVM_HOME"
test "${JAVA_HOME-}" = "$original_java_home"

# An isolated launcher must resolve first without exposing other JDK commands.
case ":$PATH:" in
  *":$GRAALVM_HOME/bin:"*)
    echo "Activation exposed the JDK bin directory" >&2
    exit 1
    ;;
esac
test "$(command -v native-image)" = "$GRAALVM_HOME/native-image-bin/native-image"
native-image --version

echo "proto integration checks passed"
