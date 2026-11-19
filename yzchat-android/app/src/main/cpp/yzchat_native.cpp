#include <jni.h>
#include <android/log.h>

#define LOG_TAG "yzchat-native"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

// Dummy native implementation - actual Rust code is loaded via JNI from libyzchat_ffi.so
extern "C" JNIEXPORT void JNICALL
Java_org_yzchat_native_YzchatNative_initialize(JNIEnv* env, jobject thiz) {
    LOGI("yzchat-native: initialize called (Rust core loaded via JNI)");
}

extern "C" JNIEXPORT void JNICALL
Java_org_yzchat_native_YzchatNative_shutdown(JNIEnv* env, jobject thiz) {
    LOGI("yzchat-native: shutdown called");
}

extern "C" JNIEXPORT jboolean JNICALL
Java_org_yzchat_native_YzchatNative_isInitialized(JNIEnv* env, jobject thiz) {
    return JNI_TRUE;
}