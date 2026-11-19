cmake_minimum_required(VERSION 3.22.1)
project("yzchat-native" LANGUAGES C CXX)

# This CMakeLists.txt is used for the native library interface
# The actual Rust library is built via cargo-ndk and copied to jniLibs
# This file exists for Gradle's externalNativeBuild to work properly

# Include directories
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Since we use prebuilt Rust libraries via JNI, we just create a dummy library
# that satisfies the NDK build system
add_library(yzchat-native SHARED yzchat_native.cpp)

# Find log library for Android logging
find_library(log-lib log)

# Link libraries
target_link_libraries(yzchat-native ${log-lib})