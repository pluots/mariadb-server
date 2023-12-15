# Rust CMake configuration
#
# This file is in charge of creating bridge libraries that Rust can link to, as
# well as invoking Cargo to build Rust plugins.

##### General configuration #####

set(rust_dir "${CMAKE_CURRENT_SOURCE_DIR}")
set(cargo_target_dir "${CMAKE_BINARY_DIR}/rust_target")
message(STATUS "rust dir ${rust_dir}")
message(STATUS "cargo target ${cargo_target_dir}")


##### Configure bridge libraries #####
# Some plugin types only have a C++ interface but not a C interface. It is
# easier to just expose a pure C interface that Rust can use, rather than
# replicating a C++ class virtual instance in Rust.
#
# This is referred to as the "bridge"

add_definitions(-DMYSQL_SERVER=1)

# Add common include directories
include_directories(
  ../include 
  ../sql
  ../libmariadb/include
  bridge
  ${PCRE_INCLUDES}
  ${SSL_INCLUDE_DIRS}
  ${ZLIB_INCLUDE_DIR}
)
            
# Bridge library for storage engine pluigns
add_convenience_library(rust_bridge_storage
  bridge/handler_bridge.cc
  ../sql/handler.cc
)

set_target_properties(
  rust_bridge_storage PROPERTIES
  LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/rust/bridge
)

# target_include_directories(
#   rust_bridge_storage PUBLIC
#   ${CMAKE_SOURCE_DIR}/rust/bridge
# )

# # Dummy target used to aggregate linker args. I can't get target_link_libraries
# # to work for some reason.
# # add_executable(rustbar bridge/m.cc)
# add_library(test_dummy_target SHARED bridge/handler_bridge.cc)
# # target_link_libraries(test_dummy_target rust_bridge_storage)
# target_include_directories(
#   test_dummy_target PUBLIC
#   ../include 
#   ../sql
#   ../libmariadb/include
#   bridge
#   ${PCRE_INCLUDES}
#   ${SSL_INCLUDE_DIRS}
#   ${ZLIB_INCLUDE_DIR}
# )
# if(MSVC OR CMAKE_SYSTEM_NAME MATCHES AIX)
#   target_link_libraries(test_dummy_target server)
# elseif(NOT CMAKE_SYSTEM_NAME STREQUAL "Linux")
#   target_link_libraries(test_dummy_target mariadbd)
# endif()
# set_target_properties(test_dummy_target PROPERTIES C_COMPILER_LAUNCHER "${CMAKE_SOURCE_DIR}/rust/test.py;c")
# set_target_properties(test_dummy_target PROPERTIES CXX_COMPILER_LAUNCHER "${CMAKE_SOURCE_DIR}/rust/test.py;cxx")
# set_target_properties(test_dummy_target PROPERTIES C_LINKER_LAUNCHER "${CMAKE_SOURCE_DIR}/rust/test.py;ld")
# set_target_properties(test_dummy_target PROPERTIES CXX_LINKER_LAUNCHER "${CMAKE_SOURCE_DIR}/rust/test.py;ld")

set(rustc_args ${rustc_args} "-Lnative=${CMAKE_CURRENT_BINARY_DIR}/sql")

# Linker args specific to storage engine plugins
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:rust_bridge_storage>")

# FIXME: I'm sure we don't actually need all of these (would be nice if the
# linker could ignore unused+undefined symbols) and I'm sure they aren't all
# compiled as PIC. Maybe we can clean this up somehow?
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:aria>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:dbug>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:myisam>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:myisammrg>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:mysys_ssl>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:mysys>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:perfschema>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:sql>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:sql_builtins>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:sql_sequence>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:strings>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:tpool>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:type_uuid>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:type_geom>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:auth_socket>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:userstat>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:user_variables>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:innobase>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:partition>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:type_inet>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:csv>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:feedback>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:heap>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:vio>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:wsrep_api_v26>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:wsrep>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:sequence>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:thread_pool_info>")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=$<TARGET_FILE:wsrep-lib>")

# System libraries
# set(rustc_args_storage ${rustc_args_storage} "-lstatic=openssl")
set(rustc_args_storage ${rustc_args_storage} "-lstatic=c++")
set(rustc_args_storage ${rustc_args_storage} "-lstatic=z")
set(rustc_args_storage ${rustc_args_storage} "-lstatic=pcre2-8")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=${OPENSSL_SSL_LIBRARY}")
set(rustc_args_storage ${rustc_args_storage} "-Clink-arg=${OPENSSL_CRYPTO_LIBRARY}")


##### Configure Rust plugins #####

# TODO: just parse any needed configuration from cargo.toml

# There are some things that are just easier to not do in CMake 
execute_process(
  COMMAND python3 "${rust_dir}/cmake-helper.py"
  OUTPUT_VARIABLE helper_output
  COMMAND_ERROR_IS_FATAL ANY
)
message(STATUS "plugin: ${helper_output}")

# Load a dictionary-like-thing in the only way cmake can
# All set variables start with "from_helper"
foreach(key_value_string ${helper_output})
  # Find the key then the value
  string(STRIP ${key_value_string} key_value_string)
  string(REGEX MATCH "^[^=]+" key ${key_value_string})
  string(REPLACE "${key}=" "" value ${key_value_string})
  # We use `|` in place of `;` for values that are lists
  string(REPLACE "|" ";" value ${value})
  # Set the variable. I wish there were a better way to handle this...
  set(${key} "${value}")
endforeach()

set(rust_shared_suffix ${CMAKE_SHARED_LIBRARY_SUFFIX})

if(APPLE)
  set(rust_shared_suffix ".dylib")
endif()

# See cmake-helper.py for the output that we get here. We loop through each
# plugin
foreach(plugin_name IN LISTS from_helper_all_plugins)
  message(STATUS "configuring rust plugin ${plugin_name}")
  set(cache_name ${from_helper_plugin_${plugin_name}_cache_name})
  set(cargo_name ${from_helper_plugin_${plugin_name}_cargo_name})
  set(target_name ${from_helper_plugin_${plugin_name}_cmake_target_name})
  set(staticlib_name "${CMAKE_STATIC_LIBRARY_PREFIX}${plugin_name}${CMAKE_STATIC_LIBRARY_SUFFIX}")
  set(dylib_name "${CMAKE_SHARED_LIBRARY_PREFIX}${plugin_name}${rust_shared_suffix}")

  # Copied from plugin.cmake, set default `howtobuild`
  if(ARG_DISABLED)
    set(howtobuild NO)
  elseif(compat STREQUAL ".")
    set(howtobuild DYNAMIC)
  elseif(compat STREQUAL "with.")
    if(NOT ARG_MODULE_ONLY)
      set(howtobuild STATIC)
    ELSE()
      set(howtobuild DYNAMIC)
    endif()
  elseif(compat STREQUAL ".without")
    set(howtobuild NO)
  elseif(compat STREQUAL "with.without")
    set(howtobuild STATIC)
  else()
    set(howtobuild DYNAMIC)
  endif()


  # NO - not at all
  # YES - static if possible, otherwise dynamic if possible, otherwise abort
  # AUTO - static if possible, otherwise dynamic, if possible
  # STATIC - static if possible, otherwise not at all
  # DYNAMIC - dynamic if possible, otherwise not at all
  set(${cache_name} ${howtobuild} CACHE STRING
    "How to build plugin ${plugin_name}. Options are: NO STATIC DYNAMIC YES AUTO.")

  if(NOT ${cache_name} MATCHES "^(NO|YES|AUTO|STATIC|DYNAMIC)$")
    message(FATAL_ERROR "Invalid value '${${cache_name}}' for ${cache_name}")
  endif()

  set(cargo_cmd 
    cargo rustc
    --target-dir=${cargo_target_dir}
    --package=${cargo_name}
    --locked
    --quiet
  )

  set(output_path "${cargo_target_dir}/release")
  
  # Configure debug/release options
  if(CMAKE_BUILD_TYPE MATCHES "Debug")
    set(cargo_cmd ${cargo_cmd} --profile=dev)
    set(output_path "${cargo_target_dir}/debug")
  elseif(CMAKE_BUILD_TYPE MATCHES "Release")
    set(cargo_cmd ${cargo_cmd} --profile=release)
  elseif(CMAKE_BUILD_TYPE MATCHES "RelWithDebInfo")
    set(cargo_cmd ${cargo_cmd} --profile=release)
  elseif(CMAKE_BUILD_TYPE MATCHES "MinSizeRel")
    set(cargo_cmd ${cargo_cmd} --profile=release)
    set(rustc_args ${rustc_args} -C strip=debuginfo)
  endif()


  set(dylib_path "${output_path}/${dylib_name}")

  # Used by build.rs
  set(env_args -E env
      CMAKE_SOURCE_DIR="${CMAKE_SOURCE_DIR}"
      CMAKE_BINARY_DIR="${CMAKE_BINARY_DIR}"
  )

  if(NOT ARG_MODULE_OUTPUT_NAME)
    if(ARG_STORAGE_ENGINE)
      set(ARG_MODULE_OUTPUT_NAME "ha_${target_name}")
    else()
      set(ARG_MODULE_OUTPUT_NAME "${target_name}")
    endif()
  endif()

  if(
    ${cache_name} MATCHES "(STATIC|AUTO|YES)" AND NOT ARG_MODULE_ONLY
    AND NOT ARG_CLIENT
  )

    # Build a staticlib
    if(CMAKE_GENERATOR MATCHES "Makefiles|Ninja")
      # If there is a shared library from previous shared build,
      # remove it. This is done just for mysql-test-run.pl 
      # so it does not try to use stale shared lib as plugin 
      # in test.
      file(REMOVE 
        ${CMAKE_CURRENT_BINARY_DIR}/${ARG_MODULE_OUTPUT_NAME}${CMAKE_SHARED_MODULE_SUFFIX})
    endif()

    set(rustc_args_this_plugin "")
    
    if(${from_helper_plugin_${plugin_name}_needs_storage})
      message(STATUS "linking in storage module")
      set(rustc_args_this_plugin ${rustc_args_this_plugin} ${rustc_args_storage})
    endif()

    add_custom_target(${target_name}
      # We set make_static_lib to generate the correct symbols
      # equivalent of `COMPILE_DEFINITIONS "MYSQL_DYNAMIC_PLUGIN$...` for C plugin_config
      # Todos:
      # TARGET_LINK_LIBRARIES (${target} mysqlservices ${ARG_LINK_LIBRARIES})
      COMMAND ${CMAKE_COMMAND}
      ${env_args}
      ${cargo_cmd}
      --crate-type=staticlib
      --
      ${rustc_args}
      ${rustc_args_this_plugin}
      --cfg=make_static_lib
      WORKING_DIRECTORY ${rust_dir}
      COMMENT "start cargo for ${target_name} with '${cargo_cmd}' static"
      VERBATIM
    )

    # Update mysqld dependencies
    set(MYSQLD_STATIC_PLUGIN_LIBS ${MYSQLD_STATIC_PLUGIN_LIBS} 
      ${target_name} ${ARG_LINK_LIBRARIES} CACHE INTERNAL "" FORCE)

    message("more to do here...")

  elseif(
    ${cache_name} MATCHES "(DYNAMIC|AUTO|YES)"
    AND NOT ARG_STATIC_ONLY AND NOT WITHOUT_DYNAMIC_PLUGINS
  )
    # Build a dynamiclib
    add_version_info(${target_name} MODULE SOURCES)

    # if(ARG_RECOMPILE_FOR_EMBEDDED OR ARG_STORAGE_ENGINE)
    #   if(MSVC OR CMAKE_SYSTEM_NAME MATCHES AIX)
    #     target_link_libraries(${target} server)
    #   elseif(NOT CMAKE_SYSTEM_NAME STREQUAL "Linux")
    #     target_link_libraries(${target} mariadbd)
    #   endif()
    # elseif(CMAKE_SYSTEM_NAME STREQUAL "Linux" AND NOT WITH_ASAN AND NOT WITH_TSAN AND NOT WITH_UBSAN AND NOT WITH_MSAN)
    #   target_link_libraries(${target} "-Wl,--no-undefined")
    # endif()

    add_custom_target(${target_name} ALL
      COMMAND ${CMAKE_COMMAND}
      ${env_args}
      ${cargo_cmd}
      --crate-type=cdylib
      --
      ${rustc_args}
      # FIXME: only provide storage arguments if we are buildling a storage engine
      ${rustc_args_storage}
      WORKING_DIRECTORY ${rust_dir}
      COMMENT "start cargo for ${target_name} with '${cargo_cmd}' dynamic"
      VERBATIM
    )

    # IF(CMAKE_SYSTEM_NAME MATCHES AIX)
    #   TARGET_LINK_OPTIONS(${target} PRIVATE "-Wl,-bE:${CMAKE_SOURCE_DIR}/libservices/mysqlservices_aix.def")
    # ENDIF()

    set_target_properties(${target} PROPERTIES PREFIX "")
    if(NOT ARG_CLIENT)
      set_target_properties(${target} PROPERTIES
        COMPILE_DEFINITIONS "MYSQL_DYNAMIC_PLUGIN${version_string}")
    endif()

    IF (NOT ARG_CLIENT)
    SET_TARGET_PROPERTIES (${target} PROPERTIES
      COMPILE_DEFINITIONS "MYSQL_DYNAMIC_PLUGIN${version_string}")
  
      endif()

    # add_custom_target(${target_name} ALL
    #   COMMAND echo "invoking cargo for ${target_name}"
    #   DEPENDS ${dylib_path}
    # )

    add_dependencies(${target_name} GenError)
    # add_dependencies(mariadb-plugin ${target_name})
    set_target_properties(${target_name} PROPERTIES OUTPUT_NAME "${target_name}")
      # mysql_install_targets(${target_name} DESTINATION ${INSTALL_PLUGINDIR} COMPONENT ${ARG_COMPONENT})


    if(${from_helper_plugin_${plugin_name}_is_example})
      set(plugin_pfx "example_")
    endif()

    set(dylib_name_installed "${plugin_pfx}${plugin_name}${CMAKE_SHARED_MODULE_SUFFIX}")

    install(FILES ${dylib_path} DESTINATION ${INSTALL_PLUGINDIR}
            RENAME ${dylib_name_installed} COMPONENT ${ARG_COMPONENT})
    
    if(ARG_CONFIG AND INSTALL_SYSCONF2DIR)
      install(FILES ${ARG_CONFIG} COMPONENT ${ARG_COMPONENT} DESTINATION ${INSTALL_SYSCONF2DIR})
    endif()
  else()
    message(STATUS "skipping rust plugin ${target_name}")
  endif()

  if(${from_helper_plugin_${plugin_name}_needs_any_services})
    add_dependencies(${target_name} mysqlservices)
  endif()

  if(${from_helper_plugin_${plugin_name}_needs_storage})
    add_dependencies(${target_name} rust_bridge_storage)
  endif()

  if(EXISTS "${CMAKE_CURRENT_SOURCE_DIR}/mysql-test")
    INSTALL_MYSQL_TEST("${CMAKE_CURRENT_SOURCE_DIR}/mysql-test/" "plugin/${subpath}")
  endif()

  # if(TARGET ${target_name})
  #   get_target_property(plugin_type ${target_name} TYPE)
  #   string(REPLACE "_LIBRARY" "" plugin_type ${plugin_type})
  #   set(have_target 1)
  # else()
  #   set(plugin_type)
  #   set(have_target 0)
  # endif()

  # if(ARG_STORAGE_ENGINE)
  #   ADD_FEATURE_INFO(${plugin} ${have_target} "Storage Engine ${plugin_type}")
  # elseif(ARG_CLIENT)
  #   ADD_FEATURE_INFO(${plugin} ${have_target} "Client plugin ${plugin_type}")
  # else()
  #   ADD_FEATURE_INFO(${plugin} ${have_target} "Server plugin ${plugin_type}")
  # endif()
  # endif(NOT WITHOUT_SERVER OR ARG_CLIENT)
endforeach()
