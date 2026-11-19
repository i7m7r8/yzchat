package org.yzchat.ui.call

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Card
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.icons.filled.CallEnd
import androidx.compose.material3.icons.filled.MicOff
import androidx.compose.material3.icons.filled.Mic
import androidx.compose.material3.icons.filled.VideocamOff
import androidx.compose.material3.icons.filled.Videocam
import androidx.compose.material3.icons.filled.FlipCameraAndroid
import androidx.compose.material3.icons.filled.VolumeUp
import androidx.compose.material3.icons.filled.VolumeOff
import androidx.compose.material3.icons.filled.PictureInPictureAlt
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
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
fun CallScreen(
    contactName: String,
    contactAvatar: String? = null,
    callType: CallType = CallType.Voice,
    isIncoming: Boolean = false,
    onAccept: () -> Unit,
    onDecline: () -> Unit,
    onEndCall: () -> Unit,
    onMuteToggle: (Boolean) -> Unit,
    onSpeakerToggle: (Boolean) -> Unit,
    onCameraToggle: (Boolean) -> Unit,
    onFlipCamera: () -> Unit,
    onPipToggle: () -> Unit,
    callState: CallState = CallState.Connecting
) {
    var isMuted by remember { mutableStateOf(false) }
    var isSpeakerOn by remember { mutableStateOf(false) }
    var isCameraOn by remember { mutableStateOf(callType == CallType.Video) }
    var isPip by remember { mutableStateOf(false) }

    Box(
        modifier = Modifier.fillMaxSize(),
        contentAlignment = Alignment.Center
    ) {
        // Background - video feed or gradient
        if (callType == CallType.Video) {
            // Remote video (full screen)
            Box(
                modifier = Modifier.fillMaxSize(),
                contentAlignment = Alignment.Center
            ) {
                // Placeholder for remote video
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = Color.Black,
                    shape = RoundedCornerShape(0.dp)
                ) {
                    // Remote video would be rendered here via AndroidView
                    Box(
                        modifier = Modifier.fillMaxSize(),
                        contentAlignment = Alignment.Center
                    ) {
                        Column(
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.Center
                        ) {
                            Box(
                                modifier = Modifier.size(120.dp).clip(CircleShape)
                                    .background(getAvatarColor(contactName))
                            ) {
                                Text(
                                    text = contactName.first().uppercase(),
                                    fontSize = 48.sp,
                                    fontWeight = FontWeight.Bold,
                                    color = Color.White,
                                    textAlign = androidx.compose.ui.text.TextAlign.Center,
                                    modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center)
                                )
                            }
                            androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(16.dp))
                            Text(
                                text = contactName,
                                fontSize = 28.sp,
                                fontWeight = FontWeight.Medium,
                                color = Color.White
                            )
                            androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(8.dp))
                            Text(
                                text = callState.displayName,
                                fontSize = 16.sp,
                                color = Color.White.copy(alpha = 0.8)
                            )
                        }
                    }

                    // Local preview (PiP)
                    if (!isPip && isCameraOn) {
                        Box(
                            modifier = Modifier
                                .align(Alignment.TopEnd)
                                .padding(16.dp)
                                .size(120.dp)
                                .clip(RoundedCornerShape(12.dp))
                                .background(getAvatarColor("You"))
                        ) {
                            // Local camera preview
                        }
                    }
                }
            }
        } else {
            // Voice call background
            Surface(
                modifier = Modifier.fillMaxSize(),
                color = androidx.compose.material3.MaterialTheme.colorScheme.surface,
                shape = RoundedCornerShape(0.dp)
            ) {
                Box(
                    modifier = Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    Column(
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.Center
                    ) {
                        Box(
                            modifier = Modifier.size(160.dp).clip(CircleShape)
                                .background(getAvatarColor(contactName))
                        ) {
                            Text(
                                text = contactName.first().uppercase(),
                                fontSize = 64.sp,
                                fontWeight = FontWeight.Bold,
                                color = Color.White,
                                textAlign = androidx.compose.ui.text.TextAlign.Center,
                                modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center)
                            )
                        }
                        androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(24.dp))
                        Text(
                            text = contactName,
                            fontSize = 32.sp,
                            fontWeight = FontWeight.Medium,
                            color = androidx.compose.material3.MaterialTheme.colorScheme.onSurface
                        )
                        androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(12.dp))
                        Text(
                            text = callState.displayName,
                            fontSize = 18.sp,
                            color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
                        )
                        if (callState == CallState.Connecting) {
                            androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(24.dp))
                            androidx.compose.material3.CircularProgressIndicator(
                                color = androidx.compose.material3.MaterialTheme.colorScheme.primary,
                                modifier = Modifier.size(48.dp)
                            )
                        }
                    }
                }
            }
        }

        // Incoming call buttons
        if (isIncoming && callState == CallState.Ringing) {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 48.dp, horizontal = 32.dp),
                horizontalArrangement = Arrangement.SpaceEvenly,
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Decline button
                Box(
                    modifier = Modifier.size(72.dp),
                    contentAlignment = Alignment.Center
                ) {
                    IconButton(
                        onClick = onDecline,
                        modifier = Modifier.size(72.dp)
                    ) {
                        Icon(
                            imageVector = Icons.Default.CallEnd,
                            contentDescription = "Decline",
                            tint = Color.White,
                            modifier = Modifier.size(28.dp)
                        )
                    }
                }
                .background(Color.Red, CircleShape)

                // Accept button
                Box(
                    modifier = Modifier.size(72.dp),
                    contentAlignment = Alignment.Center
                ) {
                    IconButton(
                        onClick = onAccept,
                        modifier = Modifier.size(72.dp)
                    ) {
                        Icon(
                            imageVector = if (callType == CallType.Video) Icons.Default.Videocam else Icons.Default.Call,
                            contentDescription = "Accept",
                            tint = Color.White,
                            modifier = Modifier.size(28.dp)
                        )
                    }
                }
                .background(androidx.compose.material3.MaterialTheme.colorScheme.tertiary, CircleShape)
            }
        } else {
            // Ongoing call controls
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 48.dp, horizontal = 32.dp),
                horizontalArrangement = Arrangement.SpaceEvenly,
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Mute
                ControlButton(
                    icon = if (isMuted) Icons.Default.MicOff else Icons.Default.Mic,
                    label = if (isMuted) "Muted" else "Mic",
                    isActive = isMuted,
                    onClick = { onMuteToggle(!isMuted); isMuted = !isMuted }
                )

                // End call
                Box(
                    modifier = Modifier.size(72.dp),
                    contentAlignment = Alignment.Center
                ) {
                    IconButton(
                        onClick = onEndCall,
                        modifier = Modifier.size(72.dp)
                    ) {
                        Icon(Icons.Default.CallEnd, contentDescription = "End call", tint = Color.White, modifier = Modifier.size(28.dp))
                    }
                }
                .background(Color.Red, CircleShape)

                // Speaker
                ControlButton(
                    icon = if (isSpeakerOn) Icons.Default.VolumeUp else Icons.Default.VolumeOff,
                    label = if (isSpeakerOn) "Speaker" else "Earpiece",
                    isActive = isSpeakerOn,
                    onClick = { onSpeakerToggle(!isSpeakerOn); isSpeakerOn = !isSpeakerOn }
                )

                if (callType == CallType.Video) {
                    // Camera toggle
                    ControlButton(
                        icon = if (isCameraOn) Icons.Default.Videocam else Icons.Default.VideocamOff,
                        label = if (isCameraOn) "Camera On" else "Camera Off",
                        isActive = !isCameraOn, // Inverted: active when off
                        onClick = { onCameraToggle(!isCameraOn); isCameraOn = !isCameraOn }
                    )

                    // Flip camera
                    ControlButton(
                        icon = Icons.Default.FlipCameraAndroid,
                        label = "Flip",
                        isActive = false,
                        onClick = onFlipCamera
                    )

                    // PiP
                    ControlButton(
                        icon = Icons.Default.PictureInPictureAlt,
                        label = if (isPip) "Full" : "PiP",
                        isActive = isPip,
                        onClick = { onPipToggle(); isPip = !isPip }
                    )
                }
            }
        }
    }
}

@Composable
fun ControlButton(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    label: String,
    isActive: Boolean,
    onClick: () -> Unit
) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(4.dp)
    ) {
        Box(
            modifier = Modifier.size(56.dp),
            contentAlignment = Alignment.Center
        ) {
            IconButton(
                onClick = onClick,
                modifier = Modifier.size(56.dp),
                colors = androidx.compose.material3.IconButtonDefaults.iconButtonColors(
                    containerColor = if (isActive) {
                        androidx.compose.material3.MaterialTheme.colorScheme.primary
                    } else {
                        androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.8)
                    },
                    contentColor = if (isActive) {
                        androidx.compose.material3.MaterialTheme.colorScheme.onPrimary
                    } else {
                        androidx.compose.material3.MaterialTheme.colorScheme.onSurface
                    }
                )
            ) {
                Icon(icon, contentDescription = label, modifier = Modifier.size(24.dp))
            }
        }
        .background(
            color = if (isActive) {
                androidx.compose.material3.MaterialTheme.colorScheme.primary
            } else {
                androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.8)
            },
            shape = CircleShape
        )

        Text(
            text = label,
            fontSize = 11.sp,
            color = androidx.compose.material3.MaterialTheme.colorScheme.onSurface,
            maxLines = 1,
            overflow = androidx.compose.ui.text.TextOverflow.Ellipsis
        )
    }
}

enum class CallType { Voice, Video }

enum class CallState {
    Connecting,
    Ringing,
    Connected,
    Holding,
    Ending,
    Ended,
    Failed
}

fun CallState.getDisplayName(): String {
    return when (this) {
        Connecting -> "Connecting..."
        Ringing -> "Ringing..."
        Connected -> "Connected"
        Holding -> "On hold"
        Ending -> "Ending..."
        Ended -> "Ended"
        Failed -> "Failed"
    }
}

fun getAvatarColor(name: String): Color {
    val colors = listOf(
        Color(0xFF0088CC), Color(0xFF50E3C2), Color(0xFFFF6B6B),
        Color(0xFF4ECDC4), Color(0xFF45B7D1), Color(0xFF96CEB4),
        Color(0xFFFFEAA7), Color(0xFFDDA0DD), Color(0xFF98D8C8),
    )
    colors[name.hashCode().absoluteValue % colors.size]
}