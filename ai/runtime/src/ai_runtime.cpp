#include "openxzuff_ai.h"

extern "C" openxzuff_ai_status openxzuff_ai_initialize(const char *model_path)
{
    if (model_path == nullptr || model_path[0] == '\0') {
        return OPENXZUFF_AI_INVALID_ARGUMENT;
    }
    /* No LiteRT backend is linked in this foundation commit. Fail explicitly. */
    return OPENXZUFF_AI_NOT_CONFIGURED;
}

extern "C" const char *openxzuff_ai_status_message(openxzuff_ai_status status)
{
    switch (status) {
    case OPENXZUFF_AI_OK: return "ready";
    case OPENXZUFF_AI_NOT_CONFIGURED: return "AI backend not configured";
    case OPENXZUFF_AI_INVALID_ARGUMENT: return "invalid argument";
    case OPENXZUFF_AI_RUNTIME_ERROR: return "AI runtime error";
    }
    return "unknown status";
}
