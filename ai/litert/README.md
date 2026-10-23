# Real LiteRT / LiteRT-LM host runner

This folder contains a runner for the real Google LiteRT-LM CLI. It does not
pretend that a host runtime is already ported into Gravity Kernel.

## Requirements

- A supported host with Python 3.13 and the official \`litert-cli-nightly\`
  package installed in an isolated environment.
- A non-empty, verified \`.litertlm\` model bundle.
- Enough RAM and disk for that model and its runtime.

Install the CLI in a virtual environment, following the official instructions:
https://github.com/google-ai-edge/LiteRT-CLI#installation

Then run a model bundle that has already passed its own conversion and inference
checks:

\`\`\`sh
./ai/litert/run_litertlm.sh /absolute/path/to/verified-model.litertlm \\
  "Explain a microkernel in one sentence."
\`\`\`

The command calls \`litert lm run\`; it does not simulate model output.

## Qwen2.5-0.5B status

The repository includes a separate real Qwen smoke test at
\`ai/qwen/smoke_test.py\`, using the upstream Hugging Face model and Transformers
on a supported CPU host. Run it with:

\`\`\`sh
python -m pip install torch transformers safetensors
python ai/qwen/smoke_test.py
\`\`\`

This proves Qwen inference on the host, **not** through LiteRT and not on
Gravity Kernel. A Qwen2.5-to-LiteRT-LM bundle must not be assumed compatible:
conversion, tokenizer/chat-template parity, bundle loading, and generated output
must all be tested before connecting the model to the LiteRT runner.

## Native OS boundary

Current LiteRT/LiteRT-LM host binaries are not drop-in components for the
freestanding RV64 kernel. A native OpenXzuff integration requires a supported
user-space runtime, memory/file I/O, threading or scheduling support, and a
verified architecture build. Keep host inference tools out of the kernel image.
