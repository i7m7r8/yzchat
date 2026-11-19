plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("kotlin-parcelize")
    id("com.google.devtools.ksp") version "2.0.20-1.0.14"
    id("kotlin-android")
    id("dagger.hilt.android.plugin") version "2.48"
}

android {
    namespace = "org.yzchat"
    compileSdk = 35

    defaultConfig {
        applicationId = "org.yzchat"
        minSdk = 24
        targetSdk = 35
        versionCode = 1
        versionName = "1.0.0"
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables.useSupportLibrary = true

        ndk {
            abiFilters.addAll(listOf("arm64-v8a", "x86_64"))
        }

        javaCompileOptions {
            annotationProcessorOptions {
                arguments = mapOf("ksp.incremental" to "true")
            }
        }
    }

    signingConfigs {
        create("release") {
            storeFile = file("../keystore.jks") as java.io.File
            storePassword = System.getenv("KEYSTORE_PASSWORD") ?: ""
            keyAlias = System.getenv("KEY_ALIAS") ?: ""
            keyPassword = System.getenv("KEY_PASSWORD") ?: ""
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            shrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("release")
        }
        debug {
            isDebuggable = true
            isMinifyEnabled = false
            matchingFallbacks += listOf("release")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
        freeCompilerArgs += listOf(
            "-Xopt-in=kotlin.RequiresOptIn",
            "-Xopt-in=kotlinx.coroutines.ExperimentalCoroutinesApi",
            "-Xopt-in=kotlinx.serialization.ExperimentalSerializationApi",
            "-Xopt-in=androidx.lifecycle.ExperimentalLifecycleApi",
            "-Xopt-in=androidx.compose.material3.ExperimentalMaterial3Api"
        )
    }

    buildFeatures {
        compose = true
        viewBinding = true
        dataBinding = true
    }

    composeOptions {
        kotlinCompilerExtensionVersion = "1.5.14"
    }

    packagingOptions {
        resources {
            excludes += listOf("META-INF/*.kotlin_module")
        }
        jniLibs {
            useLegacyPackaging = true
        }
    }

    externalNativeBuild {
        cmake {
            path("src/main/cpp/CMakeLists.txt")
        }
    }

    sourceSets {
        main {
            jniLibs.srcDirs("src/main/jniLibs")
        }
    }
}

dependencies {
    // Core Android
    val core_ktx_version = "1.13.1"
    val lifecycle_version = "2.8.2"
    val room_version = "2.6.1"
    val navigation_version = "2.7.7"
    val work_version = "2.9.0"
    val datastore_version = "1.1.1"

    implementation("androidx.core:core-ktx:$core_ktx_version")
    implementation("androidx.lifecycle:lifecycle-runtime-ktx:$lifecycle_version")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:$lifecycle_version")
    implementation("androidx.activity:activity-compose:1.9.0")

    // Compose BOM
    val compose_bom = platform("androidx.compose:compose-bom:2024.06.00")
    implementation(compose_bom)
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-graphics")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material3:material3-window-size-class")
    implementation("androidx.compose.runtime:runtime-livedata")
    implementation("androidx.compose.runtime:runtime-rxjava3")
    implementation("androidx.navigation:navigation-compose:$navigation_version")

    // Coroutines & Flow
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-flow:1.8.1")

    // Serialization
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.6.3")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-protobuf:1.6.3")

    // Room Database
    implementation("androidx.room:room-runtime:$room_version")
    implementation("androidx.room:room-ktx:$room_version")
    ksp("androidx.room:room-compiler:$room_version")

    // DataStore
    implementation("androidx.datastore:datastore-preferences:$datastore_version")
    implementation("androidx.datastore:datastore-core:$datastore_version")

    // WorkManager
    implementation("androidx.work:work-runtime-ktx:$work_version")

    // Coil for images
    implementation("io.coil-kt:coil-compose:2.6.0")

    // Accompanist
    implementation("com.google.accompanist:accompanist-permissions:0.32.0")
    implementation("com.google.accompanist:accompanist-systemuicontroller:0.32.0")

    // Hilt DI
    val hilt_version = "2.48"
    implementation("com.google.dagger:hilt-android:$hilt_version")
    ksp("com.google.dagger:hilt-compiler:$hilt_version")

    // Rust FFI (generated by uniffi)
    implementation(project(":yzchat-ffi"))

    // Testing
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.2.1")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.6.1")
    androidTestImplementation("androidx.compose.ui:ui-test-junit4")
    debugImplementation("androidx.compose.ui:ui-tooling")
    debugImplementation("androidx.compose.ui:ui-test-manifest")
}

kotlin {
    sourceSets {
        androidTest {
            kotlin.srcDir("src/androidTest/kotlin")
        }
    }
}

// KSP configuration
ksp {
    arg("room.schemaLocation", "$projectDir/schemas")
    arg("ksp.incremental", "true")
}

// Protobuf
protobuf {
    protoc {
        artifact = "com.google.protobuf:protoc:3.25.3"
    }
    plugins {
        id("grpc") {
            artifact = "io.grpc:protoc-gen-grpc-java:1.62.1"
        }
        id("kotlin") {
            artifact = "com.google.protobuf:protoc-gen-kotlin:1.25.3"
        }
    }
    generateProtoTasks {
        all().forEach { task ->
            task.builtins {
                remove(java)
                remove(kotlin)
            }
            task.plugins {
                id("grpc")
                id("kotlin")
            }
        }
    }
}