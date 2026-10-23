#ifndef OPENXZUFF_AI_H
#define OPENXZUFF_AI_H

#ifdef __cplusplus
extern "C" {
#endif

typedef enum openxzuff_ai_status {
    OPENXZUFF_AI_OK = 0,
    OPENXZUFF_AI_NOT_CONFIGURED = 1,
    OPENXZUFF_AI_INVALID_ARGUMENT = 2,
    OPENXZUFF_AI_RUNTIME_ERROR = 3
} openxzuff_ai_status;

/*
 * Stable C ABI boundary for a future C++ inference service.
 * Model loading/inference is intentionally not implemented yet: LiteRT
 * support and the Qwen2.5-0.5B model format must be verified first.
 */
openxzuff_ai_status openxzuff_ai_initialize(const char *model_path);
const char *openxzuff_ai_status_message(openxzuff_ai_status status);

#ifdef __cplusplus
}
#endif

#endif
