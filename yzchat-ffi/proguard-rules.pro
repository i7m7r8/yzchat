# ProGuard rules for yzchat-ffi module
# Keep JNI classes and methods
-keep class org.yzchat.ffi.** { *; }

# Keep Kotlin serialization
-keep class kotlinx.serialization.** { *; }

# Keep Room entities
-keep class org.yzchat.data.entity.** { *; }

# Keep FFI generated classes
-keep class org.yzchat.ffi.** { *; }

# Keep coroutines
-keep class kotlinx.coroutines.** { *; }

# Keep Flow
-keep class kotlinx.coroutines.flow.** { *; }

# Don't warn about missing classes from Rust
-dontwarn org.yzchat.ffi.**