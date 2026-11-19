package org.yzchat.ui.chat

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
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.icons.filled.ArrowBack
import androidx.compose.material3.icons.filled.AttachFile
import androidx.compose.material3.icons.filled.Call
import androidx.compose.material3.icons.filled.Videocam
import androidx.compose.material3.icons.filled.MoreVert
import androidx.compose.material3.icons.filled.Mic
import androidx.compose.material3.icons.filled.Send
import androidx.compose.material3.icons.filled.Image
import androidx.compose.material3.icons.filled.VideoCamera
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
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.px
import androidx.compose.foundation.layout.weight

@Composable
fun ChatScreen(
    contactName: String,
    contactAvatar: String? = null,
    isOnline: Boolean = true,
    onBackClick: () -> Unit,
    onCallClick: () -> Unit,
    onVideoClick: () -> Unit,
    onSendMessage: (String) -> Unit
) {
    var messageText by remember { mutableStateOf("") }
    val scrollState = rememberScrollState()

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        // Avatar
                        Box(
                            modifier = Modifier.size(40.dp).clip(CircleShape)
                                .background(getAvatarColor(contactName))
                        ) {
                            Text(
                                text = contactName.first().uppercase(),
                                fontSize = 18.sp,
                                fontWeight = FontWeight.Bold,
                                color = Color.White,
                                textAlign = TextAlign.Center,
                                modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center)
                            )
                        }
                        androidx.compose.foundation.layout.Spacer(modifier = Modifier.width(12.dp))
                        Column {
                            Text(
                                text = contactName,
                                fontSize = 17.sp,
                                fontWeight = FontWeight.Medium
                            )
                            Text(
                                text = if (isOnline) "online" else "offline",
                                fontSize = 12.sp,
                                color = if (isOnline) {
                                    androidx.compose.material3.MaterialTheme.colorScheme.tertiary
                                } else {
                                    androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
                                }
                            )
                        }
                    }
                },
                navigationIcon = {
                    IconButton(onClick = onBackClick) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                    }
                },
                actions = {
                    IconButton(onClick = onVideoClick) {
                        Icon(Icons.Default.Videocam, contentDescription = "Video call")
                    }
                    IconButton(onClick = onCallClick) {
                        Icon(Icons.Default.Call, contentDescription = "Voice call")
                    }
                    IconButton(onClick = { /* More options */ }) {
                        Icon(Icons.Default.MoreVert, contentDescription = "More")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surface
                )
            )
        }
    ) { innerPadding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
        ) {
            // Messages list
            LazyColumn(
                modifier = Modifier
                    .fillMaxSize()
                    .weight(1f)
                    .padding(bottom = 8.dp),
                reverseLayout = true,
                verticalArrangement = Arrangement.spacedBy(4.dp),
                contentPadding = androidx.compose.foundation.layout.PaddingValues(horizontal = 12.dp, vertical = 8.dp)
            ) {
                items(messages) { message ->
                    MessageBubble(message)
                }
            }

            Divider(
                color = androidx.compose.material3.MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3)
            )

            // Input area
            MessageInput(
                messageText = messageText,
                onMessageTextChange = { messageText = it },
                onSendClick = {
                    if (messageText.isNotBlank()) {
                        onSendMessage(messageText)
                        messageText = ""
                    }
                },
                onAttachClick = { /* Attach file */ },
                onMicClick = { /* Voice message */ }
            )
        }
    }
}

@Composable
fun MessageBubble(message: MessageItem) {
    val isOutgoing = message.isOutgoing
    val bubbleColor = if (isOutgoing) {
        androidx.compose.material3.MaterialTheme.colorScheme.primary
    } else {
        androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant
    }
    val textColor = if (isOutgoing) {
        androidx.compose.material3.MaterialTheme.colorScheme.onPrimary
    } else {
        androidx.compose.material3.MaterialTheme.colorScheme.onSurface
    }
    val timeColor = if (isOutgoing) {
        androidx.compose.material3.MaterialTheme.colorScheme.onPrimary.copy(alpha = 0.6)
    } else {
        androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
    }

    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = if (isOutgoing) Arrangement.End else Arrangement.Start,
        verticalAlignment = Alignment.Bottom
    ) {
        if (!isOutgoing) {
            // Avatar for incoming
            Box(
                modifier = Modifier
                    .size(32.dp)
                    .clip(CircleShape)
                    .background(getAvatarColor(message.senderName))
                    .padding(bottom = 2.dp)
            ) {
                Text(
                    text = message.senderName.first().uppercase(),
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Bold,
                    color = Color.White,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center)
                )
            }
            androidx.compose.foundation.layout.Spacer(modifier = Modifier.width(4.dp))
        }

        Column(
            crossAxisAlignment = if (isOutgoing) Alignment.End else Alignment.Start
        ) {
            Surface(
                shape = RoundedCornerShape(16.dp).copy(
                    bottomLeft = if (isOutgoing) 16.dp else 0.dp,
                    bottomRight = if (isOutgoing) 0.dp else 16.dp
                ),
                color = bubbleColor,
                modifier = Modifier
                    .wrapContentSize()
                    .padding(vertical = 2.dp)
                    .constrainMaxWidth(280.dp)
            ) {
                Column(
                    modifier = Modifier.padding(12.dp)
                ) {
                    // Sender name for incoming in groups
                    if (!isOutgoing && message.showSenderName) {
                        Text(
                            text = message.senderName,
                            fontSize = 12.sp,
                            fontWeight = FontWeight.Medium,
                            color = textColor.copy(alpha = 0.8)
                        )
                        androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(2.dp))
                    }

                    // Message content
                    when (message.type) {
                        MessageType.Text -> {
                            Text(
                                text = message.text,
                                fontSize = 15.sp,
                                color = textColor,
                                lineHeight = 22.sp
                            )
                        }
                        MessageType.Image -> {
                            // Image placeholder
                            Box(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .aspectRatio(16f / 9f)
                                    .background(Color.Gray.copy(alpha = 0.3))
                                    .clip(RoundedCornerShape(8.dp))
                            ) {
                                Icon(Icons.Default.Image, contentDescription = "Image", tint = textColor.copy(alpha = 0.5), modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center))
                            }
                        }
                        MessageType.Video -> {
                            Box(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .aspectRatio(16f / 9f)
                                    .background(Color.Gray.copy(alpha = 0.3))
                                    .clip(RoundedCornerShape(8.dp))
                            ) {
                                Icon(Icons.Default.VideoCamera, contentDescription = "Video", tint = textColor.copy(alpha = 0.5), modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center))
                            }
                        }
                        MessageType.File -> {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Icon(Icons.Default.AttachFile, contentDescription = "File", tint = textColor, modifier = Modifier.padding(end = 8.dp))
                                Column {
                                    Text(text = message.text, fontSize = 14.sp, color = textColor)
                                    Text(text = message.fileInfo ?: "", fontSize = 12.sp, color = textColor.copy(alpha = 0.7))
                                }
                            }
                        }
                        MessageType.Voice -> {
                            Row(
                                modifier = Modifier.fillMaxWidth(),
                                verticalAlignment = Alignment.CenterVertically
                            ) {
                                Icon(Icons.Default.Mic, contentDescription = "Voice", tint = textColor, modifier = Modifier.padding(end = 8.dp))
                                // Waveform placeholder
                                Box(
                                    modifier = Modifier
                                        .width(120.dp)
                                        .height(32.dp)
                                        .background(textColor.copy(alpha = 0.2))
                                        .clip(RoundedCornerShape(16.dp))
                                )
                                Text(text = message.voiceDuration ?: "0:00", fontSize = 14.sp, color = textColor, modifier = Modifier.padding(start = 8.dp))
                            }
                        }
                        MessageType.Call -> {
                            Text(
                                text = message.text,
                                fontSize = 13.sp,
                                fontWeight = FontWeight.Medium,
                                color = textColor
                            )
                        }
                    }

                    // Time and status
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.End,
                        verticalAlignment = Alignment.Bottom
                    ) {
                        Text(
                            text = message.time,
                            fontSize = 11.sp,
                            color = timeColor
                        )
                        if (isOutgoing) {
                            androidx.compose.foundation.layout.Spacer(modifier = Modifier.width(4.dp))
                            Icon(
                                imageVector = when (message.status) {
                                    MessageStatus.Sent -> Icons.Default.Check
                                    MessageStatus.Delivered -> Icons.Default.DoneAll
                                    MessageStatus.Read -> Icons.Default.DoneAll
                                    else -> Icons.Default.Check
                                },
                                contentDescription = null,
                                tint = if (message.status == MessageStatus.Read) {
                                    androidx.compose.material3.MaterialTheme.colorScheme.tertiary
                                } else {
                                    timeColor
                                },
                                modifier = Modifier.size(14.dp)
                            )
                        }
                    }
                }
            }
        }

        if (isOutgoing) {
            androidx.compose.foundation.layout.Spacer(modifier = Modifier.width(4.dp))
            // Avatar for outgoing (optional, usually not shown)
        }
    }
}

@Composable
fun MessageInput(
    messageText: String,
    onMessageTextChange: (String) -> Unit,
    onSendClick: () -> Unit,
    onAttachClick: () -> Unit,
    onMicClick: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        IconButton(onClick = onAttachClick) {
            Icon(Icons.Default.AttachFile, contentDescription = "Attach")
        }

        androidx.compose.foundation.layout.Spacer(modifier = Modifier.width(4.dp))

        androidx.compose.material3.TextField(
            value = messageText,
            onValueChange = onMessageTextChange,
            modifier = Modifier
                .fillMaxWidth()
                .weight(1f)
                .height(48.dp)
                .padding(start = 8.dp, end = 8.dp),
            placeholder = { Text("Message", color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant) },
            singleLine = false,
            maxLines = 5,
            colors = androidx.compose.material3.TextFieldDefaults.textFieldColors(
                containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5),
                focusedContainerColor = androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant,
                unfocusedContainerColor = androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5)
            ),
            shape = RoundedCornerShape(24.dp)
        )

        androidx.compose.foundation.layout.Spacer(modifier = Modifier.width(4.dp))

        if (messageText.isBlank()) {
            IconButton(onClick = onMicClick) {
                Icon(Icons.Default.Mic, contentDescription = "Voice message",
                    tint = androidx.compose.material3.MaterialTheme.colorScheme.primary)
            }
        } else {
            IconButton(onClick = onSendClick) {
                Icon(Icons.Default.Send, contentDescription = "Send",
                    tint = androidx.compose.material3.MaterialTheme.colorScheme.primary)
            }
        }
    }
}

// Data classes
data class MessageItem(
    val id: String,
    val senderName: String,
    val text: String,
    val time: String,
    val isOutgoing: Boolean,
    val type: MessageType = MessageType.Text,
    val status: MessageStatus = MessageStatus.Sent,
    val showSenderName: Boolean = false,
    val fileInfo: String? = null,
    val voiceDuration: String? = null
)

enum class MessageType { Text, Image, Video, File, Voice, Call }
enum class MessageStatus { Sending, Sent, Delivered, Read, Failed }

// Sample messages
val messages = listOf(
    MessageItem("1", "Alice", "Hey! How are you?", "10:30", false),
    MessageItem("2", "You", "I'm doing great! Thanks for asking.", "10:31", true, status = MessageStatus.Read),
    MessageItem("3", "Alice", "That's awesome to hear 😊", "10:32", false),
    MessageItem("4", "You", "How about you?", "10:32", true, status = MessageStatus.Read),
    MessageItem("5", "Alice", "Pretty good! Just finished a project.", "10:33", false),
    MessageItem("6", "Alice", "IMG_2024.jpg", "10:34", false, type = MessageType.Image),
    MessageItem("7", "You", "Nice photo! Where was this taken?", "10:35", true, status = MessageStatus.Delivered),
)

// Helper function
fun getAvatarColor(name: String): Color {
    val colors = listOf(
        Color(0xFF0088CC), Color(0xFF50E3C2), Color(0xFFFF6B6B),
        Color(0xFF4ECDC4), Color(0xFF45B7D1), Color(0xFF96CEB4),
        Color(0xFFFFEAA7), Color(0xFFDDA0DD), Color(0xFF98D8C8),
    )
    colors[name.hashCode().absoluteValue % colors.size]
}