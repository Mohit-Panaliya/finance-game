package com.fintrack

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.fintrack.ui.theme.FintrackTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        ServerService.start(this)
        setContent {
            FintrackTheme {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.background
                ) {
                    MainScreen()
                }
            }
        }
    }
}

@Composable
fun MainScreen() {
    val context = LocalContext.current
    var isServerRunning by remember { mutableStateOf(RustServer.isRunning()) }
    Column(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(text = "Fintrack", style = MaterialTheme.typography.headlineLarge)
        Spacer(modifier = Modifier.height(8.dp))
        Text(text = if (isServerRunning) "Server running" else "Server stopped", style = MaterialTheme.typography.bodyLarge)
        Spacer(modifier = Modifier.height(24.dp))
        Button(onClick = {
            if (isServerRunning) {
                ServerService.stop(context)
            } else {
                ServerService.start(context)
            }
            isServerRunning = RustServer.isRunning()
        }) {
            Text(if (isServerRunning) "Stop Server" else "Start Server")
        }
        Spacer(modifier = Modifier.height(16.dp))
        Text(text = "Server: http://127.0.0.1:5155", style = MaterialTheme.typography.bodySmall)
    }
}
