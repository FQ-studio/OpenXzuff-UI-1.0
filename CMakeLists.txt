cmake_minimum_required(VERSION 3.20)
project(OpenXzuffUI VERSION 1.0.0 LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
enable_testing()

# Gravity AI: custom knowledge store, BM25 RAG and native quantized inference.
add_library(gravity-ai-native ai/gravity/src/knowledge.cpp ai/gravity/src/shared_knowledge_pack.cpp ai/gravity/src/gail.cpp)
target_include_directories(gravity-ai-native PUBLIC ai/gravity/include)
target_compile_options(gravity-ai-native PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
add_executable(gravity-ai-test ai/gravity/tests/gravity_ai_test.cpp)
target_link_libraries(gravity-ai-test PRIVATE gravity-ai-native)
target_compile_options(gravity-ai-test PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-UNDEBUG;-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME gravity-ai-sql-rag-native-inference COMMAND gravity-ai-test)

add_executable(gravity-gail-test ai/gravity/tests/gail_arithmetic_test.cpp)
target_link_libraries(gravity-gail-test PRIVATE gravity-ai-native)
target_compile_options(gravity-gail-test PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-UNDEBUG;-Wall;-Wextra;-Wpedantic;-Werror>)
add_test(NAME gravity-gail-arithmetic-sentence COMMAND gravity-gail-test)

add_executable(gravity-elf-to-gnb ai/gravity/tools/elf_to_gnb.cpp)
target_compile_options(gravity-elf-to-gnb PRIVATE $<$<CXX_COMPILER_ID:GNU,Clang>:-Wall;-Wextra;-Wpedantic;-Werror>)
