package org.yzchat

import android.app.Application
import android.content.Context
import dagger.hilt.android.HiltAndroidApp
import org.yzchat.di.DatabaseModule
import org.yzchat.di.RepositoryModule
import org.yzchat.ffi.YzchatCore
import org.yzchat.ffi.CoreConfig

@HiltAndroidApp
class YzchatApplication : Application() {

    private var yzchatCore: YzchatCore? = null

    override fun onCreate() {
        super.onCreate()
        initializeYzchatCore()
    }

    private fun initializeYzchatCore() {
        val dataDir = filesDir.absolutePath
        val config = CoreConfig(
            dataDir = dataDir,
            enableLogging = BuildConfig.DEBUG,
            logLevel = if (BuildConfig.DEBUG) "debug" else "info",
            autoAcceptCalls = false,
            autoDownloadMedia = true
        )

        yzchatCore = YzchatCore.new(config)
        yzchatCore?.initialize()

        // Set up global error handling
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            android.util.Log.e("Yzchat", "Uncaught exception in ${thread.name}", throwable)
        }
    }

    fun getYzchatCore(): YzchatCore? = yzchatCore

    companion object {
        @Suppress("UNUSED_PARAMETER")
        fun getInstance(context: Context): YzchatApplication {
            return context.applicationContext as YzchatApplication
        }
    }
}