package org.yzchat.ui.main

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.ListItemHeadline
import androidx.compose.material3.ListItemLeading
import androidx.compose.material3.ListItemTrailing
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.icons.Icons
import androidx.compose.material3.icons.filled.Chat
import androidx.compose.material3.icons.filled.ContactPhone
import androidx.compose.material3.icons.filled.Settings
import androidx.compose.material3.icons.filled.Call
import androidx.compose.material3.icons.filled.Videocam
import androidx.compose.material3.icons.filled.Add
import androidx.compose.material3.icons.filled.Search
import androidx.compose.material3.icons.filled.MoreVert
import androidx.compose.material3.icons.filled.Menu
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.unit.px
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.ui.Alignment
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.graphics.Canvas
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Paint
import androidx.compose.ui.graphics.drawscope.drawCircle
import androidx.compose.ui.unit.Dp

@Composable
fun MainScreen(
    onChatClick: (String) -> Unit,
    onNewChatClick: () -> Unit,
    onCallClick: () -> Unit,
    onSettingsClick: () -> Unit
) {
    var selectedTab by remember { mutableStateOf(0) }
    val scrollState = rememberScrollState()

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Text(
                        text = when (selectedTab) {
                            0 -> "Chats"
                            1 -> "Calls"
                            2 -> "Contacts"
                            else -> "Settings"
                        },
                        fontWeight = FontWeight.Medium,
                        fontSize = 20.sp
                    )
                },
                navigationIcon = {
                    if (selectedTab == 0) {
                        IconButton(onClick = { /* Open drawer */ }) {
                            Icon(Icons.Default.Menu, contentDescription = "Menu")
                        }
                    }
                },
                actions = {
                    when (selectedTab) {
                        0 -> {
                            IconButton(onClick = { onNewChatClick() }) {
                                Icon(Icons.Default.Add, contentDescription = "New chat")
                            }
                            IconButton(onClick = { /* Search */ }) {
                                Icon(Icons.Default.Search, contentDescription = "Search")
                            }
                            IconButton(onClick = { /* More */ }) {
                                Icon(Icons.Default.MoreVert, contentDescription = "More")
                            }
                        }
                        1 -> {
                            IconButton(onClick = { /* Call log */ }) {
                                Icon(Icons.Default.Call, contentDescription = "Call log")
                            }
                        }
                        2 -> {
                            IconButton(onClick = { /* Add contact */ }) {
                                Icon(Icons.Default.Add, contentDescription = "Add contact")
                            }
                        }
                        3 -> {
                            // Settings actions
                        }
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surface,
                    titleContentColor = androidx.compose.material3.MaterialTheme.colorScheme.onSurface
                )
            )
        },
        bottomBar = {
            NavigationBar(
                modifier = Modifier.padding(bottom = 0.dp),
                containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surface
            ) {
                val tabs = listOf(
                    TabItem(0, "Chats", Icons.Default.Chat),
                    TabItem(1, "Calls", Icons.Default.Call),
                    TabItem(2, "Contacts", Icons.Default.ContactPhone),
                    TabItem(3, "Settings", Icons.Default.Settings)
                )
                tabs.forEach { tab ->
                    NavigationBarItem(
                        icon = { Icon(tab.icon, contentDescription = null) },
                        label = { Text(tab.label, fontSize = 12.sp) },
                        selected = selectedTab == tab.index,
                        onClick = { selectedTab = tab.index },
                        selectedContentColor = androidx.compose.material3.MaterialTheme.colorScheme.primary,
                        unselectedContentColor = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }
    ) { innerPadding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(innerPadding)
        ) {
            when (selectedTab) {
                0 -> ChatsTab(onChatClick = onChatClick, onNewChatClick = onNewChatClick)
                1 -> CallsTab()
                2 -> ContactsTab()
                3 -> SettingsTab(onSettingsClick = onSettingsClick)
            }
        }
    }
}

data class TabItem(
    val index: Int,
    val label: String,
    val icon: androidx.compose.material3.icons.Icons.Filled.(()->androidx.compose.ui.graphics.vector.ImageVector)
)

@Composable
fun ChatsTab(
    onChatClick: (String) -> Unit,
    onNewChatClick: () -> Unit
) {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(0.dp)
    ) {
        // Sample chat list - replace with real data from ViewModel
        listOf(
            ChatItem("Alice", "Hey, how are you?", "10:30", 2, true),
            ChatItem("Bob", "Meeting at 3pm", "09:15", 0, false),
            ChatItem("Charlie", "📷 Photo", "Yesterday", 0, true),
            ChatItem("Diana", "Voice message • 0:45", "Monday", 1, false),
            ChatItem("Eve", "Document.pdf", "Sunday", 0, true),
        ).forEach { chat ->
            ChatListItem(chat, onClick = { onChatClick(chat.name) })
            Divider(
                modifier = Modifier.padding(start = 72.dp),
                color = androidx.compose.material3.MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3)
            )
        }

        // Floating action button for new chat
        androidx.compose.material3.FloatingActionButton(
            onClick = onNewChatClick,
            modifier = Modifier
                .align(androidx.compose.foundation.layout.Alignment.BottomEnd)
                .padding(16.dp),
            containerColor = androidx.compose.material3.MaterialTheme.colorScheme.primary,
            contentColor = androidx.compose.material3.MaterialTheme.colorScheme.onPrimary
        ) {
            Icon(Icons.Default.Add, contentDescription = "New chat")
        }
    }
}

@Composable
fun ChatListItem(
    chat: ChatItem,
    onClick: () -> Unit
) {
    ListItem(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth(),
        leadingContent = {
            ListItemLeading(
                modifier = Modifier.size(56.dp)
            ) {
                // Avatar
                Box(
                    modifier = Modifier
                        .size(56.dp)
                        .clip(CircleShape)
                        .background(getAvatarColor(chat.name))
                ) {
                    Text(
                        text = chat.name.first().uppercase(),
                        fontSize = 24.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.White,
                        textAlign = androidx.compose.ui.text.TextAlign.Center,
                        modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center)
                    )
                }
            }
        },
        headlineContent = {
            Column {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = androidx.compose.foundation.layout.Arrangement.SpaceBetween
                ) {
                    ListItemHeadline(
                        headlineContent = { Text(chat.name, fontWeight = FontWeight.Medium, fontSize = 16.sp) }
                    )
                    Text(
                        text = chat.time,
                        fontSize = 12.sp,
                        color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(2.dp))
                ListItemDefaults.ListItemSecondaryText(
                    text = chat.message,
                    maxLines = 1,
                    overflow = androidx.compose.ui.text.TextOverflow.Ellipsis,
                    color = if (chat.unreadCount > 0) {
                        androidx.compose.material3.MaterialTheme.colorScheme.onSurface
                    } else {
                        androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
                    },
                    fontWeight = if (chat.unreadCount > 0) FontWeight.Medium else FontWeight.Normal
                )
            }
        },
        trailingContent = {
            if (chat.unreadCount > 0) {
                ListItemTrailing {
                    Box(
                        modifier = Modifier
                            .padding(end = 8.dp)
                            .wrapContentSize(Alignment.Center),
                        contentAlignment = Alignment.Center
                    ) {
                        // Unread badge
                        Box(
                            modifier = Modifier
                                .width(20.dp)
                                .height(20.dp)
                                .background(androidx.compose.material3.MaterialTheme.colorScheme.primary, CircleShape)
                        ) {
                            Text(
                                text = if (chat.unreadCount > 9) "9+" else chat.unreadCount.toString(),
                                fontSize = 11.sp,
                                fontWeight = FontWeight.Bold,
                                color = androidx.compose.material3.MaterialTheme.colorScheme.onPrimary,
                                textAlign = androidx.compose.ui.text.TextAlign.Center,
                                modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center)
                            )
                        }
                    }
                }
            }
        },
        colors = ListItemDefaults.listItemColors(
            containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surface,
            hoveredContainerColor = androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.5),
            pressedContainerColor = androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3)
        )
    )
}

data class ChatItem(
    val name: String,
    val message: String,
    val time: String,
    val unreadCount: Int,
    val isMuted: Boolean
)

@Composable
fun CallsTab() {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(0.dp)
    ) {
        // Call log
        listOf(
            CallLogItem("Alice", "Outgoing", "10:30", CallType.Video, false),
            CallLogItem("Bob", "Missed", "09:15", CallType.Voice, true),
            CallLogItem("Charlie", "Incoming", "Yesterday", CallType.Video, false),
            CallLogItem("Diana", "Outgoing", "Monday", CallType.Voice, false),
        ).forEach { call ->
            CallListItem(call)
            Divider(
                modifier = Modifier.padding(start = 72.dp),
                color = androidx.compose.material3.MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3)
            )
        }
    }
}

@Composable
fun CallListItem(call: CallLogItem) {
    ListItem(
        onClick = { /* Call back */ },
        modifier = Modifier.fillMaxWidth(),
        leadingContent = {
            ListItemLeading(
                modifier = Modifier.size(56.dp)
            ) {
                Box(
                    modifier = Modifier
                        .size(56.dp)
                        .clip(CircleShape)
                        .background(getAvatarColor(call.name))
                ) {
                    Icon(
                        imageVector = when (call.type) {
                            CallType.Video -> Icons.Default.Videocam
                            CallType.Voice -> Icons.Default.Call
                        },
                        contentDescription = null,
                        tint = Color.White,
                        modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center)
                    )
                }
            }
        },
        headlineContent = {
            Column {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = androidx.compose.foundation.layout.Arrangement.SpaceBetween
                ) {
                    ListItemHeadline(
                        headlineContent = { Text(call.name, fontWeight = FontWeight.Medium, fontSize = 16.sp) }
                    )
                    Text(
                        text = call.time,
                        fontSize = 12.sp,
                        color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
                androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(2.dp))
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = androidx.compose.foundation.layout.Arrangement.SpaceBetween
                ) {
                    Text(
                        text = call.direction,
                        fontSize = 14.sp,
                        color = if (call.isMissed) Color.Red else androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
                    )
                    Icon(
                        imageVector = when (call.type) {
                            CallType.Video -> Icons.Default.Videocam
                            CallType.Voice -> Icons.Default.Call
                        },
                        contentDescription = null,
                        tint = if (call.isMissed) Color.Red else androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(16.dp)
                    )
                }
            }
        },
        colors = ListItemDefaults.listItemColors(
            containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surface
        )
    )
}

data class CallLogItem(
    val name: String,
    val direction: String,
    val time: String,
    val type: CallType,
    val isMissed: Boolean
)

enum class CallType { Voice, Video }

@Composable
fun ContactsTab() {
    Column(
        modifier = Modifier.fillMaxSize(),
        verticalArrangement = Arrangement.spacedBy(0.dp)
    ) {
        // Contacts list
        listOf(
            ContactItem("Alice Johnson", "+1 555-0101", true),
            ContactItem("Bob Smith", "+1 555-0102", false),
            ContactItem("Charlie Brown", "+1 555-0103", true),
            ContactItem("Diana Prince", "+1 555-0104", false),
        ).forEach { contact ->
            ContactListItem(contact)
            Divider(
                modifier = Modifier.padding(start = 72.dp),
                color = androidx.compose.material3.MaterialTheme.colorScheme.outlineVariant.copy(alpha = 0.3)
            )
        }
    }
}

@Composable
fun ContactListItem(contact: ContactItem) {
    ListItem(
        onClick = { /* Open chat */ },
        modifier = Modifier.fillMaxWidth(),
        leadingContent = {
            ListItemLeading(
                modifier = Modifier.size(56.dp)
            ) {
                Box(
                    modifier = Modifier
                        .size(56.dp)
                        .clip(CircleShape)
                        .background(getAvatarColor(contact.name))
                ) {
                    Text(
                        text = contact.name.first().uppercase(),
                        fontSize = 24.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.White,
                        textAlign = androidx.compose.ui.text.TextAlign.Center,
                        modifier = Modifier.fillMaxSize().wrapContentSize(Alignment.Center)
                    )
                }
            }
        },
        headlineContent = {
            Column {
                ListItemHeadline(
                    headlineContent = { Text(contact.name, fontWeight = FontWeight.Medium, fontSize = 16.sp) }
                )
                androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(2.dp))
                Text(
                    text = contact.phone,
                    fontSize = 14.sp,
                    color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        },
        trailingContent = {
            if (contact.isOnline) {
                ListItemTrailing {
                    Box(
                        modifier = Modifier
                            .size(12.dp)
                            .background(androidx.compose.material3.MaterialTheme.colorScheme.tertiary, CircleShape)
                    )
                }
            }
        },
        colors = ListItemDefaults.listItemColors(
            containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surface
        )
    )
}

data class ContactItem(
    val name: String,
    val phone: String,
    val isOnline: Boolean
)

@Composable
fun SettingsTab(onSettingsClick: () -> Unit) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        SettingsSection("Account") {
            SettingItem("Profile", "Edit name, photo, bio") { /* Navigate */ }
            SettingItem("Privacy & Security", "Block users, 2FA, sessions") { /* Navigate */ }
            SettingItem("Phone Number", "Change number") { /* Navigate */ }
        }

        SettingsSection("Chat Settings") {
            SettingItem("Chat Background", "Wallpapers, themes") { /* Navigate */ }
            SettingItem("Auto-Download Media", "Photos, videos, documents") { /* Navigate */ }
            SettingItem("Data & Storage", "Usage, cache, proxy") { /* Navigate */ }
        }

        SettingsSection("Notifications") {
            SettingItem("Notification Settings", "Sounds, vibration, preview") { /* Navigate */ }
            SettingItem("Chat Notifications", "Private chats, groups, channels") { /* Navigate */ }
        }

        SettingsSection("Calls") {
            SettingItem("Voice & Video Calls", "Quality, peer-to-peer") { /* Navigate */ }
            SettingItem("Screen Sharing", "Resolution, frame rate") { /* Navigate */ }
        }

        SettingsSection("Advanced") {
            SettingItem("Experimental Settings", "Beta features") { /* Navigate */ }
            SettingItem("Debug Logs", "Export logs") { /* Navigate */ }
            SettingItem("About", "Version, licenses, legal") { /* Navigate */ }
        }
    }
}

@Composable
fun SettingsSection(title: String, content: @Composable () -> Unit) {
    Column(
        modifier = Modifier.fillMaxWidth(),
        verticalArrangement = Arrangement.spacedBy(0.dp)
    ) {
        Text(
            text = title,
            fontSize = 12.sp,
            fontWeight = FontWeight.Medium,
            color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(vertical = 8.dp, start = 4.dp)
        )
        Surface(
            modifier = Modifier.fillMaxWidth(),
            shape = androidx.compose.material3.MaterialTheme.shapes.medium,
            color = androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3),
            tonalElevation = 0.dp
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(0.dp)) {
                content()
            }
        }
    }
}

@Composable
fun SettingItem(title: String, subtitle: String, onClick: () -> Unit) {
    ListItem(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth(),
        headlineContent = {
            ListItemHeadline(
                headlineContent = { Text(title, fontWeight = FontWeight.Medium, fontSize = 16.sp) }
            )
        },
        supportingContent = {
            Text(text = subtitle, fontSize = 14.sp, color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant)
        },
        trailingContent = {
            ListItemTrailing {
                Icon(Icons.Default.ChevronRight, contentDescription = null, tint = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant)
            }
        },
        colors = ListItemDefaults.listItemColors(
            containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3)
        )
    )
}

// Helper function for avatar colors
fun getAvatarColor(name: String): Color {
    val colors = listOf(
        Color(0xFF0088CC), Color(0xFF50E3C2), Color(0xFFFF6B6B),
        Color(0xFF4ECDC4), Color(0xFF45B7D1), Color(0xFF96CEB4),
        Color(0xFFFFEAA7), Color(0xFFDDA0DD), Color(0xFF98D8C8),
        Color(0xFFF7DC6F), Color(0xFFBB8FCE), Color(0xFF85C1E9)
    )
    val index = name.hashCode().absoluteValue % colors.size
    colors[index]
}