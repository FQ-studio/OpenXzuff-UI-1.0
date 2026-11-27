if(NOT DEFINED XGO OR NOT DEFINED WORK_DIR OR NOT DEFINED PROJECT_PARENT)
  message(FATAL_ERROR "XGO and WORK_DIR are required")
endif()

file(REMOVE_RECURSE "${WORK_DIR}")
file(MAKE_DIRECTORY "${PROJECT_PARENT}")
execute_process(COMMAND "${XGO}" create CliSmoke WORKING_DIRECTORY "${PROJECT_PARENT}"
  RESULT_VARIABLE create_result OUTPUT_VARIABLE create_out ERROR_VARIABLE create_err)
if(NOT create_result EQUAL 0)
  message(FATAL_ERROR "xgo create failed (${create_result})\n${create_out}\n${create_err}")
endif()

execute_process(COMMAND "${XGO}" check WORKING_DIRECTORY "${WORK_DIR}"
  RESULT_VARIABLE check_result OUTPUT_VARIABLE check_out ERROR_VARIABLE check_err)
if(NOT check_result EQUAL 0)
  message(FATAL_ERROR "xgo check failed (${check_result})\n${check_out}\n${check_err}")
endif()
if(NOT check_out MATCHES "Xzuff Go check: PASS")
  message(FATAL_ERROR "xgo did not forward Coder Checker output\n${check_out}\n${check_err}")
endif()

execute_process(COMMAND "${XGO}" test WORKING_DIRECTORY "${WORK_DIR}"
  RESULT_VARIABLE test_result OUTPUT_VARIABLE test_out ERROR_VARIABLE test_err)
if(NOT test_result EQUAL 0)
  message(FATAL_ERROR "xgo test failed (${test_result})\n${test_out}\n${test_err}")
endif()

execute_process(COMMAND "${XGO}" run WORKING_DIRECTORY "${WORK_DIR}"
  RESULT_VARIABLE run_result OUTPUT_VARIABLE run_out ERROR_VARIABLE run_err)
if(run_result EQUAL 0)
  message(FATAL_ERROR "xgo run unexpectedly claimed that an application runtime is available")
endif()
if(NOT run_err MATCHES "general .xexe process loader")
  message(FATAL_ERROR "xgo run did not explain the missing process loader\n${run_out}\n${run_err}")
endif()

execute_process(COMMAND "${XGO}" build WORKING_DIRECTORY "${WORK_DIR}"
  RESULT_VARIABLE build_result OUTPUT_VARIABLE build_out ERROR_VARIABLE build_err)
if(NOT build_result EQUAL 0)
  message(FATAL_ERROR "xgo build failed (${build_result})\n${build_out}\n${build_err}")
endif()

if(NOT EXISTS "${WORK_DIR}/build/CliSmoke.xexe")
  message(FATAL_ERROR "xgo build did not create the expected .xexe")
endif()

file(WRITE "${WORK_DIR}/src/main.xgo" "function main() returns i32 {\n    return 2147483648;\n}\n")
execute_process(COMMAND "${XGO}" check WORKING_DIRECTORY "${WORK_DIR}"
  RESULT_VARIABLE bad_check_result OUTPUT_VARIABLE bad_check_out ERROR_VARIABLE bad_check_err)
if(bad_check_result EQUAL 0)
  message(FATAL_ERROR "Coder Checker accepted an out-of-range i32 unexpectedly")
endif()
if(bad_check_result EQUAL 2)
  message(FATAL_ERROR "xgo check returned usage exit 2 instead of a source diagnostic\n${bad_check_out}\n${bad_check_err}")
endif()

message(STATUS "xgo CLI smoke test: PASS (create/check/test/run guard/build and checker rejection)")
file(REMOVE_RECURSE "${WORK_DIR}")
