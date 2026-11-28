// Top-level build file where you can add configuration options common to all sub-projects/modules.

plugins {
    id("com.android.application") version "8.5.0" apply false
    id("org.jetbrains.kotlin.android") version "1.9.23" apply false
}

allprojects {
    repositories {
        google()
        mavenCentral()
    }

    configurations.all {
        resolutionStrategy.foreachDependency { details ->
            if (details.requested.group == "androidx.core") {
                details.useVersion("1.12.0")
            }
        }
    }
}

tasks.register("clean", Delete::class) {
    delete(rootProject.buildDir)
}
