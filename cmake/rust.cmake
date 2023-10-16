# TODO: just parse any needed configuration from cargo.toml
# CMAKE_PARSE_ARGUMENTS(ARG
# "STORAGE_ENGINE;STATIC_ONLY;MODULE_ONLY;MANDATORY;DEFAULT;DISABLED;NOT_EMBEDDED;RECOMPILE_FOR_EMBEDDED;CLIENT"
# "MODULE_OUTPUT_NAME;STATIC_OUTPUT_NAME;COMPONENT;CONFIG;VERSION"
# "LINK_LIBRARIES;DEPENDS"
# ${ARGN}
# )

# BUG: for some reason this doesn't always invoke cargo to recompile

macro(CONFIGURE_RUST_PLUGINS)
  set(rust_dir "${CMAKE_SOURCE_DIR}/rust")
  set(cargo_target_dir "${CMAKE_CURRENT_BINARY_DIR}/rust_target")
  message(STATUS "rust dir ${rust_dir}")

  execute_process(COMMAND python3 "${rust_dir}/cmake_helper.py" OUTPUT_VARIABLE plugins)

  # Add common include directories
  INCLUDE_DIRECTORIES(${CMAKE_SOURCE_DIR}/include 
                    ${CMAKE_SOURCE_DIR}/sql
                    ${PCRE_INCLUDES}
                    ${SSL_INCLUDE_DIRS}
                    ${ZLIB_INCLUDE_DIR})

  # find_library(servlib NAMES "services")
  # message("LIBPATH ${CMAKE_LIBRARY_PATH} FINDLIBS ${servlib}")

  # See cmake_helper.py for the output that we get here. We loop through each
  # plugin
  foreach(entry IN LISTS plugins)
    string(REPLACE "|" ";" entry "${entry}")
    list(GET entry 0 cache_name)
    list(GET entry 1 target_name)
    list(GET entry 2 cargo_name)
    list(GET entry 3 staticlib_name)
    list(GET entry 4 dylib_name_out)
    list(GET entry 5 dylib_name_final)
    set(output_path "${cargo_target_dir}/release")

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
      "How to build plugin ${cargo_name}. Options are: NO STATIC DYNAMIC YES AUTO.")

    if(NOT ${${cache_name}} MATCHES "^(NO|YES|AUTO|STATIC|DYNAMIC)$")
      message(FATAL_ERROR "Invalid value ${cache_name} for ${cache_name}")
    endif()

    set(cargo_cmd 
      cargo rustc
      --target-dir=${cargo_target_dir}
      --package=${cargo_name}
      --locked
      --quiet
    )

    set(rustc_extra_args -L "native=${CMAKE_CURRENT_BINARY_DIR}/libservices")

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
      set(rustc_extra_args ${rustc_extra_args} -C strip=debuginfo)
    endif()

    set(dylib_path "${output_path}/${dylib_name_out}")

    # Used by build.rs
    set(env_args -E env CMAKE_SOURCE_DIR=${CMAKE_SOURCE_DIR}
        CMAKE_BINARY_DIR=${CMAKE_BINARY_DIR}
    )

    if(NOT ARG_MODULE_OUTPUT_NAME)
      if(ARG_STORAGE_ENGINE)
        set(ARG_MODULE_OUTPUT_NAME "ha_${target_name}")
      else()
        set(ARG_MODULE_OUTPUT_NAME "${target_name}")
      endif()
    endif()

    if(
      ${${cache_name}} MATCHES "(STATIC|AUTO|YES)" AND NOT ARG_MODULE_ONLY
      AND NOT ARG_CLIENT
    )
    message(STATUS "configuring rust plugin ${target_name} as static")

      # Build a staticlib
      if(CMAKE_GENERATOR MATCHES "Makefiles|Ninja")
        # If there is a shared library from previous shared build,
        # remove it. This is done just for mysql-test-run.pl 
        # so it does not try to use stale shared lib as plugin 
        # in test.
        file(REMOVE 
          ${CMAKE_CURRENT_BINARY_DIR}/${ARG_MODULE_OUTPUT_NAME}${CMAKE_SHARED_MODULE_SUFFIX})
      endif()


      add_custom_target(${target_name}
        # We set make_static_lib to generate the correct symbols
        # equivalent of `COMPILE_DEFINITIONS "MYSQL_DYNAMIC_PLUGIN$...` for C plugins
        # Todos:
        # TARGET_LINK_LIBRARIES (${target} mysqlservices ${ARG_LINK_LIBRARIES})
        COMMAND ${CMAKE_COMMAND} ${env_args}
          ${cargo_cmd} --crate-type=staticlib
          -- ${rustc_extra_args} --cfg=make_static_lib
        WORKING_DIRECTORY ${rust_dir}
        COMMENT "start cargo for ${target_name} with '${cargo_cmd}' static"
        VERBATIM
      )

      # add_custom_target(${target_name} ALL
      #   COMMAND echo "invoking cargo for ${target_name}"
      #   DEPENDS ${staticlib_name}
      # )

      # Update mysqld dependencies
      SET (MYSQLD_STATIC_PLUGIN_LIBS ${MYSQLD_STATIC_PLUGIN_LIBS} 
        ${target_name} ${ARG_LINK_LIBRARIES} CACHE INTERNAL "" FORCE)

      message("more to do here...")

    elseif(
      ${${cache_name}} MATCHES "(DYNAMIC|AUTO|YES)"
      AND NOT ARG_STATIC_ONLY AND NOT WITHOUT_DYNAMIC_PLUGINS
    )
      # Build a dynamiclib
      message(STATUS "configuring rust plugin ${target_name} as dynamic")

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
          ${rustc_extra_args}
        WORKING_DIRECTORY ${rust_dir}
        COMMENT "start cargo for ${target_name} with '${cargo_cmd}' dynamic"
        VERBATIM
      )

      add_dependencies(${target_name} mysqlservices)

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
    ENDIF()

      # add_custom_target(${target_name} ALL
      #   COMMAND echo "invoking cargo for ${target_name}"
      #   DEPENDS ${dylib_path}
      # )

      add_dependencies(${target_name} GenError)
      # add_dependencies(mariadb-plugin ${target_name})
      set_target_properties(${target_name} PROPERTIES OUTPUT_NAME "${target_name}")
        # mysql_install_targets(${target_name} DESTINATION ${INSTALL_PLUGINDIR} COMPONENT ${ARG_COMPONENT})
      install(FILES ${dylib_path} DESTINATION ${INSTALL_PLUGINDIR} RENAME ${dylib_name_final} COMPONENT ${ARG_COMPONENT})
      
      if(ARG_CONFIG AND INSTALL_SYSCONF2DIR)
        install(FILES ${ARG_CONFIG} COMPONENT ${ARG_COMPONENT} DESTINATION ${INSTALL_SYSCONF2DIR})
      endif()
    else()
      message(STATUS "skipping rust plugin ${target_name}")
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
endmacro()
