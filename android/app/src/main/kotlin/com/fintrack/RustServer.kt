package com.fintrack

import android.content.Context
import java.io.File
import java.io.FileOutputStream

object RustServer {
    init {
        System.loadLibrary("finance_game")
    }

    @JvmStatic
    external fun startServer(dataDir: String, port: Int): Int

    @JvmStatic
    external fun stopServer(): Int

    @JvmStatic
    external fun isRunning(): Boolean

    @JvmStatic
    external fun getStatus(): String

    fun extractBinary(context: Context): File {
        val outFile = File(context.filesDir, "finance-game-cli")
        if (!outFile.exists()) {
            context.assets.open("finance-game-cli").use { input ->
                FileOutputStream(outFile).use { output ->
                    input.copyTo(output)
                }
            }
            outFile.setExecutable(true)
        }
        return outFile
    }
}
