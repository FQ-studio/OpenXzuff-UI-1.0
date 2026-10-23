#!/usr/bin/env python3
"""Run a real Qwen2.5-0.5B-Instruct CPU inference smoke test on a supported host.

This does not claim that Qwen runs inside the bare-metal Gravity Kernel.
Model weights are downloaded from Hugging Face at runtime and are not committed.
"""
import argparse
import sys

import torch
from transformers import AutoModelForCausalLM, AutoTokenizer


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model", default="Qwen/Qwen2.5-0.5B-Instruct")
    parser.add_argument(
        "--prompt",
        default="Reply in one short sentence: what is a microkernel?",
    )
    parser.add_argument("--max-new-tokens", type=int, default=48)
    args = parser.parse_args()

    if args.max_new_tokens < 1 or args.max_new_tokens > 512:
        parser.error("--max-new-tokens must be between 1 and 512")

    print(f"Loading tokenizer: {args.model}", flush=True)
    tokenizer = AutoTokenizer.from_pretrained(args.model, trust_remote_code=False)
    print(f"Loading model on CPU: {args.model}", flush=True)
    model = AutoModelForCausalLM.from_pretrained(
        args.model,
        torch_dtype=torch.float32,
        trust_remote_code=False,
    )
    model.eval()

    messages = [{"role": "user", "content": args.prompt}]
    inputs = tokenizer.apply_chat_template(
        messages,
        add_generation_prompt=True,
        tokenize=True,
        return_dict=True,
        return_tensors="pt",
    )

    print("Running actual model inference...", flush=True)
    with torch.inference_mode():
        output_ids = model.generate(
            **inputs,
            max_new_tokens=args.max_new_tokens,
            do_sample=False,
        )

    prompt_length = inputs["input_ids"].shape[-1]
    answer = tokenizer.decode(
        output_ids[0][prompt_length:],
        skip_special_tokens=True,
    ).strip()
    if not answer:
        print("FAIL: model returned no generated text", file=sys.stderr)
        return 1

    print("QWEN_INFERENCE_OK")
    print(answer)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
