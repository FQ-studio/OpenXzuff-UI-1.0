#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 /path/to/verified-model.litertlm [prompt]" >&2
  exit 2
fi

model_path="$1"
if [[ $# -eq 2 ]]; then
  prompt="$2"
else
  prompt="Explain a microkernel in one sentence."
fi

if ! command -v litert >/dev/null 2>&1; then
  echo "ERROR: LiteRT CLI is not installed. See ai/litert/README.md." >&2
  exit 127
fi
if [[ ! -s "$model_path" ]]; then
  echo "ERROR: model file is missing or empty: $model_path" >&2
  exit 66
fi

# Runs the real LiteRT-LM engine. This is a host-side tool, not a Gravity
# bare-metal service; the model must already be converted and verified.
exec litert lm run "$model_path" --prompt "$prompt"
