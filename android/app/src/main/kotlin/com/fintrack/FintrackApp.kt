package com.fintrack

import android.app.Application
import androidx.work.Configuration

class FintrackApp : Application(), Configuration.Provider {
    override fun onCreate() {
        super.onCreate()
        // Start server on app creation
        ServerService.start(this)
    }

    override val workManagerConfiguration: Configuration
        get() = Configuration.Builder()
            .setMinimumLoggingLevel(android.util.Log.INFO)
            .build()
}
