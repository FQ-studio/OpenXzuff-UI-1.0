# Gravity AI — Native Knowledge and Inference

Gravity AI is an in-tree OpenXzuff UI subsystem implemented in C++20. It is not a separate Core edition.

## Implemented now
- Custom embedded knowledge store with a small SQL surface (CREATE TABLE knowledge and SELECT ... FROM knowledge), document insertion, bounded binary persistence, duplicate-ID checks and corruption checks.
- RAG retrieval using BM25 lexical ranking over indexed document text.
- Native CPU inference primitive using int8-quantized linear weights, float activations, tanh output and greedy logit selection.
- No PyTorch, Python runtime or external database dependency.

## Honest model boundary
The inference runtime is executable C++ code, but it is not itself a trained language model. A trained/exported checkpoint and tokenizer are required for useful natural-language generation. Current RAG is lexical BM25, not embedding-based semantic retrieval. These limitations are explicit.

## Knowledge sharing and automatic updates
The store accepts knowledge documents from trusted importers and persists them in .gxk format. The next integration stage is a source-adapter pipeline for repository documentation, curated public technical references and user-authorized ChatGPT/API exports, retaining source attribution, version, timestamp and licensing metadata. There is no supported API for reading GPT-6 private weights or internal model state, so no such transfer is claimed.

## Build and test
The top-level CMake build produces gravity-ai-test; CTest runs the SQL/RAG/inference smoke test. This implementation currently targets host C++20 for CI validation. RV64-native deployment still requires integration with the OS runtime, allocator, storage and scheduler.
