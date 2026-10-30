package com.fintrack

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import java.io.File
import java.io.FileOutputStream

class ServerService : Service() {

    companion object {
        const val CHANNEL_ID = "fintrack_server"
        const val NOTIFICATION_ID = 1001
        const val EXTRA_PORT = "port"

        fun start(context: Context, port: Int = 5155) {
            val intent = Intent(context, ServerService::class.java).apply {
                putExtra(EXTRA_PORT, port)
            }
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                context.startForegroundService(intent)
            } else {
                context.startService(intent)
            }
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, ServerService::class.java))
        }
    }

    private var port: Int = 5155
    private var process: Process? = null

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        port = intent?.getIntExtra(EXTRA_PORT, 5155) ?: 5155
        val notification = buildNotification()
        startForeground(NOTIFICATION_ID, notification)

        Thread {
            startServer()
        }.start()

        return START_STICKY
    }

    private fun startServer() {
        try {
            val binary = extractBinary()
            val dataDir = filesDir.absolutePath
            val dbPath = "$dataDir/finance_game.sqlite"

            val pb = ProcessBuilder(
                binary.absolutePath,
                "start",
                "-e", "development",
                "-p", port.toString()
            )
            pb.directory(filesDir)
            pb.environment()["DATABASE_URL"] = "sqlite://$dataDir/finance_game_development_loco.sqlite?mode=rwc"
            pb.environment()["PORT"] = port.toString()
            pb.environment()["BINDING"] = "0.0.0.0"
            pb.environment()["LOG_LEVEL"] = "info"
            pb.redirectErrorStream(true)

            process = pb.start()

            // Read output
            process?.inputStream?.bufferedReader()?.forEachLine { line ->
                android.util.Log.d("FintrackServer", line)
            }
        } catch (e: Exception) {
            android.util.Log.e("FintrackServer", "Failed to start server", e)
        }
    }

    private fun extractBinary(): File {
        val outFile = File(filesDir, "finance-game-cli")
        if (!outFile.exists()) {
            assets.open("finance-game-cli").use { input ->
                FileOutputStream(outFile).use { output ->
                    input.copyTo(output)
                }
            }
            outFile.setExecutable(true)
        }
        return outFile
    }

    override fun onDestroy() {
        super.onDestroy()
        process?.destroy()
        process = null
    }

    override fun onBind(intent: Intent?): IBinder? = null

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Fintrack Server",
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Keeps the Fintrack server running"
                setShowBadge(false)
            }
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }

    private fun buildNotification(): Notification {
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            Intent(this, MainActivity::class.java),
            PendingIntent.FLAG_IMMUTABLE or PendingIntent.FLAG_UPDATE_CURRENT
        )
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("Fintrack Server")
            .setContentText("Running on port $port (0.0.0.0)")
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .setSilent(true)
            .build()
    }
}
