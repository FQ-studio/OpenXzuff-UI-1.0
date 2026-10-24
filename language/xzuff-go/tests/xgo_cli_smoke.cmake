if(NOT DEFINED XGO OR NOT DEFINED WORK_DIR)
  message(FATAL_ERROR "XGO and WORK_DIR are required")
endif()

file(REMOVE_RECURSE "\${WORK_DIR}")
execute_process(COMMAND "\${XGO}" create CliSmoke WORKING_DIRECTORY "\${CMAKE_CURRENT_BINARY_DIR}"
  RESULT_VARIABLE create_result OUTPUT_VARIABLE create_out ERROR_VARIABLE create_err)
if(NOT create_result EQUAL 0)
  message(FATAL_ERROR "xgo create failed (\${create_result})\n\${create_out}\n\${create_err}")
endif()

execute_process(COMMAND "\${XGO}" check WORKING_DIRECTORY "\${WORK_DIR}"
  RESULT_VARIABLE check_result OUTPUT_VARIABLE check_out ERROR_VARIABLE check_err)
if(NOT check_result EQUAL 0)
  message(FATAL_ERROR "xgo check failed (\${check_result})\n\${check_out}\n\${check_err}")
endif()

execute_process(COMMAND "\${XGO}" run WORKING_DIRECTORY "\${WORK_DIR}"
  RESULT_VARIABLE run_result OUTPUT_VARIABLE run_out ERROR_VARIABLE run_err)
if(NOT run_result EQUAL 0)
  message(FATAL_ERROR "xgo run failed (\${run_result})\n\${run_out}\n\${run_err}")
endif()

execute_process(COMMAND "\${XGO}" build WORKING_DIRECTORY "\${WORK_DIR}"
  RESULT_VARIABLE build_result OUTPUT_VARIABLE build_out ERROR_VARIABLE build_err)
if(NOT build_result EQUAL 0)
  message(FATAL_ERROR "xgo build failed (\${build_result})\n\${build_out}\n\${build_err}")
endif()

if(NOT EXISTS "\${WORK_DIR}/build/CliSmoke.xexe")
  message(FATAL_ERROR "xgo build did not create the expected .xexe")
endif()

file(WRITE "\${WORK_DIR}/src/main.xgo" "function main() returns i32 {\n    return 2147483648;\n}\n")
execute_process(COMMAND "\${XGO}" check WORKING_DIRECTORY "\${WORK_DIR}"
  RESULT_VARIABLE bad_check_result OUTPUT_VARIABLE bad_check_out ERROR_VARIABLE bad_check_err)
if(bad_check_result EQUAL 0)
  message(FATAL_ERROR "Coder Checker accepted an out-of-range i32 unexpectedly")
endif()
if(bad_check_result EQUAL 2)
  message(FATAL_ERROR "xgo check returned usage exit 2 instead of a source diagnostic\n\${bad_check_out}\n\${bad_check_err}")
endif()

message(STATUS "xgo CLI smoke test: PASS (create/check/run/build and checker rejection)")
file(REMOVE_RECURSE "\${WORK_DIR}")
