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
import androidx.compose.material3.Icon
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.icons.filled.ArrowForward
import androidx.compose.material3.icons.filled.Check
import androidx.compose.material3.icons.filled.Close
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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

@Composable
fun OnboardingScreen(onFinish: () -> Unit) {
    var currentPage by remember { mutableStateOf(0) }
    val pages = remember { onboardingPages }

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
            // Page indicator
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 32.dp),
                horizontalArrangement = Arrangement.Center
            ) {
                pages.indices.forEach { index ->
                    Box(
                        modifier = Modifier
                            .size(if (index == currentPage) 24.dp else 8.dp, 8.dp)
                            .padding(horizontal = 4.dp)
                            .background(
                                color = if (index == currentPage) {
                                    androidx.compose.material3.MaterialTheme.colorScheme.primary
                                } else {
                                    androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.3)
                                },
                                shape = RoundedCornerShape(4.dp)
                            )
                            .animateContentSize()
                    )
                }
            }

            // Page content
            val page = pages[currentPage]
            Column(
                modifier = Modifier.fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(24.dp)
            ) {
                Box(
                    modifier = Modifier.size(160.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        imageVector = page.icon,
                        contentDescription = null,
                        tint = androidx.compose.material3.MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(80.dp)
                    )
                }
                .background(
                    color = androidx.compose.material3.MaterialTheme.colorScheme.primaryContainer,
                    shape = CircleShape
                )

                Text(
                    text = page.title,
                    fontSize = 24.sp,
                    fontWeight = FontWeight.Bold,
                    color = androidx.compose.material3.MaterialTheme.colorScheme.onSurface,
                    textAlign = TextAlign.Center
                )

                Text(
                    text = page.description,
                    fontSize = 16.sp,
                    color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant,
                    textAlign = TextAlign.Center,
                    lineHeight = 24.sp
                )
            }

            androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(32.dp))

            // Navigation buttons
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween
            ) {
                if (currentPage > 0) {
                    Button(
                        onClick = { currentPage-- },
                        colors = androidx.compose.material3.ButtonDefaults.buttonColors(
                            containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant,
                            contentColor = androidx.compose.material3.MaterialTheme.colorScheme.onSurface
                        )
                    ) {
                        Text("Back", fontWeight = FontWeight.Medium)
                    }
                } else {
                    androidx.compose.foundation.layout.Spacer(modifier = Modifier.width(80.dp))
                }

                if (currentPage < pages.size - 1) {
                    Button(
                        onClick = { currentPage++ },
                        colors = androidx.compose.material3.ButtonDefaults.buttonColors(
                            containerColor = androidx.compose.material3.MaterialTheme.colorScheme.primary,
                            contentColor = androidx.compose.material3.MaterialTheme.colorScheme.onPrimary
                        )
                    ) {
                        Row(
                            horizontalArrangement = Arrangement.Center,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text("Next", fontWeight = FontWeight.Medium)
                            androidx.compose.foundation.layout.Spacer(modifier = Modifier.width(8.dp))
                            Icon(Icons.Default.ArrowForward, contentDescription = null)
                        }
                    }
                } else {
                    Button(
                        onClick = onFinish,
                        colors = androidx.compose.material3.ButtonDefaults.buttonColors(
                            containerColor = androidx.compose.material3.MaterialTheme.colorScheme.primary,
                            contentColor = androidx.compose.material3.MaterialTheme.colorScheme.onPrimary
                        )
                    ) {
                        Row(
                            horizontalArrangement = Arrangement.Center,
                            verticalAlignment = Alignment.CenterVertically
                        ) {
                            Text("Get Started", fontWeight = FontWeight.Medium)
                            androidx.compose.foundation.layout.Spacer(modifier = Modifier.width(8.dp))
                            Icon(Icons.Default.Check, contentDescription = null)
                        }
                    }
                }
            }
        }
    }
}

data class OnboardingPage(
    val icon: androidx.compose.ui.graphics.vector.ImageVector,
    val title: String,
    val description: String
)

val onboardingPages = listOf(
    OnboardingPage(
        icon = androidx.compose.material3.icons.Icons.Default.Shield,
        title = "Quantum-Resistant Security",
        description = "Your messages are protected with ML-KEM-768 + X25519 hybrid encryption and ML-DSA-65 + Ed25519 signatures. Even quantum computers can't break it."
    ),
    OnboardingPage(
        icon = androidx.compose.material3.icons.Icons.Default.Router,
        title = "True Peer-to-Peer",
        description = "No servers, no cloud, no metadata collection. Messages travel directly between devices using Iroh's QUIC-based P2P network."
    ),
    OnboardingPage(
        icon = androidx.compose.material3.icons.Icons.Default.Videocam,
        title = "4K Video Calls",
        description = "Crystal clear AV1 video at 4K60, Opus audio at 510kbps, and 4K30 screen sharing. All encrypted end-to-end."
    ),
    OnboardingPage(
        icon = androidx.compose.material3.icons.Icons.Default.Folder,
        title = "File Sharing",
        description = "Send files of any size with streaming transfer, encryption, and resume support. No size limits."
    ),
    OnboardingPage(
        icon = androidx.compose.material3.icons.Icons.Default.Group,
        title = "Groups & Channels",
        description = "Create groups with up to 200,000 members. Admin tools, polls, reactions, and more."
    ),
    OnboardingPage(
        icon = androidx.compose.material3.icons.Icons.Default.Settings,
        title = "Your Data, Your Rules",
        description = "Open source, self-hosted options, local-first storage. You own your keys, your data, your identity."
    )
)