package org.yzchat.ui.onboarding

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.Icon
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.icons.filled.ArrowForward
import androidx.compose.material3.icons.filled.Lock
import androidx.compose.material3.icons.filled.Shield
import androidx.compose.material3.icons.filled.Speed
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.Alignment
import androidx.compose.ui.graphics.Paint
import androidx.compose.ui.graphics.drawscope.drawCircle
import androidx.compose.ui.unit.px

@Composable
fun WelcomeScreen(onGetStarted: () -> Unit) {
    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(32.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            // App icon
            Box(
                modifier = Modifier.size(120.dp).clip(CircleShape)
                    .background(androidx.compose.material3.MaterialTheme.colorScheme.primary)
            ) {
                Icon(
                    imageVector = androidx.compose.material3.icons.Icons.Default.Chat,
                    contentDescription = "yzchat",
                    tint = Color.White,
                    modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center)
                )
            }

            androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(32.dp))

            Text(
                text = "yzchat",
                fontSize = 36.sp,
                fontWeight = FontWeight.Bold,
                color = androidx.compose.material3.MaterialTheme.colorScheme.onSurface,
                textAlign = TextAlign.Center
            )

            androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(8.dp))

            Text(
                text = "Pure P2P Messenger\nQuantum-Resistant • Zero Infrastructure",
                fontSize = 16.sp,
                color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant,
                textAlign = TextAlign.Center
            )

            androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(48.dp))

            // Features
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(16.dp)
            ) {
                FeatureRow(
                    icon = Icons.Default.Shield,
                    title = "Quantum-Resistant Encryption",
                    description = "ML-KEM-768 + X25519, ML-DSA-65 + Ed25519"
                )
                FeatureRow(
                    icon = Icons.Default.Speed,
                    title = "True Peer-to-Peer",
                    description = "No servers, no domains, no hosting"
                )
                FeatureRow(
                    icon = Icons.Default.Lock,
                    title = "4K Video & Crystal Audio",
                    description = "AV1 video, Opus audio, screen share"
                )
            }

            androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(48.dp))

            Button(
                onClick = onGetStarted,
                modifier = Modifier
                    .fillMaxWidth()
                    .height(56.dp),
                colors = androidx.compose.material3.ButtonDefaults.buttonColors(
                    containerColor = androidx.compose.material3.MaterialTheme.colorScheme.primary,
                    contentColor = androidx.compose.material3.MaterialTheme.colorScheme.onPrimary
                )
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.Center,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("Get Started", fontSize = 16.sp, fontWeight = FontWeight.Medium)
                    androidx.compose.foundation.layout.Spacer(modifier = Modifier.width(8.dp))
                    Icon(Icons.Default.ArrowForward, contentDescription = null)
                }
            }

            androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(24.dp))

            Text(
                text = "Open Source • GPL-3.0 • Built with Rust",
                fontSize = 12.sp,
                color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6),
                textAlign = TextAlign.Center
            )
        }
    }
}

@Composable
fun FeatureRow(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    title: String,
    description: String
) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Box(
            modifier = Modifier.size(48.dp),
            contentAlignment = Alignment.Center
        ) {
            Icon(
                imageVector = icon,
                contentDescription = null,
                tint = androidx.compose.material3.MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(24.dp)
            )
        }
        .background(
            color = androidx.compose.material3.MaterialTheme.colorScheme.primaryContainer,
            shape = CircleShape
        )

        Column(
            modifier = Modifier.weight(1f),
            crossAxisAlignment = Alignment.Start
        ) {
            Text(
                text = title,
                fontSize = 15.sp,
                fontWeight = FontWeight.Medium,
                color = androidx.compose.material3.MaterialTheme.colorScheme.onSurface
            )
            Text(
                text = description,
                fontSize = 13.sp,
                color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
            )
        }
    }
}