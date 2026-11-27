# Gravity AI — Native Knowledge, GAIL and Inference

Gravity AI is an in-tree OpenXzuff UI subsystem implemented in C++20. It is not a separate Core edition.

## Implemented in this branch
- Existing custom embedded knowledge store, bounded .gxk binary persistence and BM25 lexical RAG.
- Existing INT8-weight native inference primitive.
- GAIL runtime entry points in C++: validates the initial @start/@ai/native/int8/code_end program envelope.
- Arithmetic Engine: integer addition, subtraction, multiplication, division (/ and ÷), unary signs, parentheses, precedence, division-by-zero rejection and signed 64-bit overflow checks.
- Sentence Engine: appends the computed answer after a generated deterministic Malay sentence. This is a template engine, not a trained natural-language generator.
- Example source: examples/arithmetic.gxk. Host CMake tests exercise the arithmetic and runtime behavior.
- No PyTorch, Python runtime or external database dependency.

## GAIL behavior and limits
The current GAIL runtime validates the program envelope and exposes arithmetic question evaluation. It does not yet parse arbitrary GAIL declarations or compile general GAIL programs to machine code. The arithmetic evaluator is deterministic C++; it is not neural inference. The `precision int8` setting applies to neural inference configuration and does not truncate exact integer arithmetic.

## Honest model boundary
The inference primitive is not a trained language model. A trained/exported checkpoint, tokenizer and generation pipeline are still required for open-ended natural-language generation. Current RAG is lexical BM25, not embedding-based semantic retrieval. SentenceEngine currently uses a deterministic template.

## Build and test
The top-level CMake build produces gravity-ai-test and gravity-gail-test. CTest covers SQL/RAG/inference primitives and GAIL arithmetic/sentence behavior. This implementation currently targets host C++20 for CI validation. RV64-native deployment still requires integration with the OS runtime, allocator, storage and scheduler.
