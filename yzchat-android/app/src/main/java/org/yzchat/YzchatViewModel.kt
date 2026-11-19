package org.yzchat

import android.app.Application
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import javax.inject.Inject
import kotlinx.coroutines.launch
import org.yzchat.ffi.YzchatCore
import org.yzchat.ffi.CoreConfig

@HiltViewModel
class YzchatViewModel @Inject constructor(
    private val application: Application
) : AndroidViewModel(application) {

    private var yzchatCore: YzchatCore? = null

    fun initializeYzchatCore() {
        val dataDir = application.filesDir.absolutePath
        val config = CoreConfig(
            dataDir = dataDir,
            enableLogging = BuildConfig.DEBUG,
            logLevel = if (BuildConfig.DEBUG) "debug" else "info",
            autoAcceptCalls = false,
            autoDownloadMedia = true
        )

        yzchatCore = YzchatCore.new(config)
        
        viewModelScope.launch {
            yzchatCore?.initialize()
        }
    }

    fun getYzchatCore(): YzchatCore? = yzchatCore

    override fun onCleared() {
        super.onCleared()
        yzchatCore?.shutdown()
    }
}