cmake_minimum_required(VERSION 3.16)
project(plotffi_example CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Option to specify the plotffi library location
set(PLOTFFI_DIR "${CMAKE_CURRENT_SOURCE_DIR}/.." CACHE PATH "Path to plotffi crate directory")

# Determine library paths based on build type and platform
if(CMAKE_BUILD_TYPE STREQUAL "Release")
    set(PLOTFFI_BUILD_DIR "${PLOTFFI_DIR}/target/release")
else()
    set(PLOTFFI_BUILD_DIR "${PLOTFFI_DIR}/target/debug")
endif()

# Create imported library target
add_library(plotffi SHARED IMPORTED GLOBAL)

if(WIN32)
    # Windows (MSVC)
    set_target_properties(plotffi PROPERTIES
        IMPORTED_LOCATION "${PLOTFFI_BUILD_DIR}/plotffi.dll"
        IMPORTED_IMPLIB "${PLOTFFI_BUILD_DIR}/plotffi.dll.lib"
        INTERFACE_INCLUDE_DIRECTORIES "${PLOTFFI_DIR}/include"
    )
elseif(APPLE)
    # macOS
    set_target_properties(plotffi PROPERTIES
        IMPORTED_LOCATION "${PLOTFFI_BUILD_DIR}/libplotffi.dylib"
        INTERFACE_INCLUDE_DIRECTORIES "${PLOTFFI_DIR}/include"
    )
else()
    # Linux
    set_target_properties(plotffi PROPERTIES
        IMPORTED_LOCATION "${PLOTFFI_BUILD_DIR}/libplotffi.so"
        INTERFACE_INCLUDE_DIRECTORIES "${PLOTFFI_DIR}/include"
    )
endif()

# Add the example executable
add_executable(plotffi_example main.cpp)

# Include the C++ wrapper header directory
target_include_directories(plotffi_example PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# Link against the plotffi library
target_link_libraries(plotffi_example PRIVATE plotffi)

# On Windows, copy DLL to output directory for easy execution
if(WIN32)
    add_custom_command(TARGET plotffi_example POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
            "${PLOTFFI_BUILD_DIR}/plotffi.dll"
            $<TARGET_FILE_DIR:plotffi_example>
    )
endif()
