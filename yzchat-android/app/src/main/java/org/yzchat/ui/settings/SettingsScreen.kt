package org.yzchat.ui.settings

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.ListItemHeadline
import androidx.compose.material3.ListItemLeading
import androidx.compose.material3.ListItemTrailing
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.icons.filled.ArrowBack
import androidx.compose.material3.icons.filled.AccountCircle
import androidx.compose.material3.icons.filled.Palette
import androidx.compose.material3.icons.filled.Notifications
import androidx.compose.material3.icons.filled.Storage
import androidx.compose.material3.icons.filled.Language
import androidx.compose.material3.icons.filled.Help
import androidx.compose.material3.icons.filled.Info
import androidx.compose.material3.icons.filled.PrivacyTip
import androidx.compose.material3.icons.filled.DataUsage
import androidx.compose.material3.icons.filled.Key
import androidx.compose.material3.icons.filled.Build
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.Alignment
import androidx.compose.ui.graphics.Paint
import androidx.compose.ui.graphics.drawscope.drawCircle
import androidx.compose.ui.unit.px

@Composable
fun SettingsScreen(onBackClick: () -> Unit) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Settings", fontWeight = FontWeight.Medium, fontSize = 20.sp) },
                navigationIcon = {
                    IconButton(onClick = onBackClick) {
                        Icon(androidx.compose.material3.icons.Icons.Default.ArrowBack, contentDescription = "Back")
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
            // Account section
            SettingsSection("Account") {
                SettingItem(
                    icon = Icons.Default.AccountCircle,
                    title = "Profile",
                    subtitle = "Name, photo, bio, username",
                    onClick = { /* Navigate */ }
                )
                Divider(modifier = Modifier.padding(start = 56.dp))
                SettingItem(
                    icon = Icons.Default.Key,
                    title = "Privacy & Security",
                    subtitle = "Block users, 2FA, active sessions, passkeys",
                    onClick = { /* Navigate */ }
                )
                Divider(modifier = Modifier.padding(start = 56.dp))
                SettingItem(
                    icon = Icons.Default.PrivacyTip,
                    title = "Encryption Keys",
                    subtitle = "View keys, verify contacts, backup identity",
                    onClick = { /* Navigate */ }
                )
            }

            // Chat settings
            SettingsSection("Chat Settings") {
                SettingItem(
                    icon = Icons.Default.Palette,
                    title = "Chat Background",
                    subtitle = "Wallpapers, themes, bubble colors",
                    onClick = { /* Navigate */ }
                )
                Divider(modifier = Modifier.padding(start = 56.dp))
                SettingItem(
                    icon = Icons.Default.DataUsage,
                    title = "Auto-Download Media",
                    subtitle = "Photos, videos, documents, on Wi-Fi/mobile",
                    onClick = { /* Navigate */ }
                )
                Divider(modifier = Modifier.padding(start = 56.dp))
                SettingItem(
                    icon = Icons.Default.Storage,
                    title = "Data & Storage",
                    subtitle = "Storage usage, cache, proxy settings",
                    onClick = { /* Navigate */ }
                )
                Divider(modifier = Modifier.padding(start = 56.dp))
                SettingItem(
                    icon = Icons.Default.Language,
                    title = "Language",
                    subtitle = "App language, translation",
                    onClick = { /* Navigate */ }
                )
            }

            // Notifications
            SettingsSection("Notifications") {
                SettingItem(
                    icon = Icons.Default.Notifications,
                    title = "Notification Settings",
                    subtitle = "Sounds, vibration, LED, preview, priority",
                    onClick = { /* Navigate */ }
                )
                Divider(modifier = Modifier.padding(start = 56.dp))
                SettingItem(
                    icon = Icons.Default.Notifications,
                    title = "Chat Notifications",
                    subtitle = "Private chats, groups, channels, mentions",
                    onClick = { /* Navigate */ }
                )
            }

            // Calls
            SettingsSection("Calls") {
                SettingItem(
                    icon = Icons.Default.Videocam,
                    title = "Voice & Video Calls",
                    subtitle = "Quality, peer-to-peer, noise suppression",
                    onClick = { /* Navigate */ }
                )
                Divider(modifier = Modifier.padding(start = 56.dp))
                SettingItem(
                    icon = Icons.Default.ScreenShare,
                    title = "Screen Sharing",
                    subtitle = "Resolution, frame rate, audio capture",
                    onClick = { /* Navigate */ }
                )
            }

            // Advanced
            SettingsSection("Advanced") {
                SettingItem(
                    icon = Icons.Default.Build,
                    title = "Experimental Features",
                    subtitle = "Beta features, debug options",
                    onClick = { /* Navigate */ }
                )
                Divider(modifier = Modifier.padding(start = 56.dp))
                SettingItem(
                    icon = Icons.Default.BugReport,
                    title = "Debug Logs",
                    subtitle = "Export logs, performance metrics",
                    onClick = { /* Navigate */ }
                )
                Divider(modifier = Modifier.padding(start = 56.dp))
                SettingItem(
                    icon = Icons.Default.Info,
                    title = "About yzchat",
                    subtitle = "Version 1.0.0 • GPL-3.0 • Built with Rust",
                    onClick = { /* Navigate */ }
                )
            }

            // Danger zone
            SettingsSection("Danger Zone") {
                SettingItem(
                    icon = Icons.Default.DeleteForever,
                    title = "Delete Account",
                    subtitle = "Permanently delete your account and all data",
                    isDestructive = true,
                    onClick = { /* Show confirmation */ }
                )
            }
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
            shape = RoundedCornerShape(12.dp),
            color = androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3),
            tonalElevation = 0.dp
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(0.dp)) {
                content()
            }
        }
        androidx.compose.foundation.layout.Spacer(modifier = Modifier.height(16.dp))
    }
}

@Composable
fun SettingItem(
    icon: androidx.compose.ui.graphics.vector.ImageVector,
    title: String,
    subtitle: String,
    isDestructive: Boolean = false,
    onClick: () -> Unit
) {
    val titleColor = if (isDestructive) Color.Red else androidx.compose.material3.MaterialTheme.colorScheme.onSurface

    ListItem(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth(),
        leadingContent = {
            ListItemLeading(
                modifier = Modifier.size(40.dp)
            ) {
                Box(
                    modifier = Modifier.size(40.dp),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        imageVector = icon,
                        contentDescription = null,
                        tint = if (isDestructive) Color.Red else androidx.compose.material3.MaterialTheme.colorScheme.primary,
                        modifier = Modifier.size(20.dp)
                    )
                }
            }
        },
        headlineContent = {
            ListItemHeadline(
                headlineContent = { Text(title, fontWeight = FontWeight.Medium, fontSize = 16.sp, color = titleColor) }
            )
        },
        supportingContent = {
            Text(text = subtitle, fontSize = 14.sp, color = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant)
        },
        trailingContent = {
            ListItemTrailing {
                Icon(
                    imageVector = androidx.compose.material3.icons.Icons.Default.ChevronRight,
                    contentDescription = null,
                    tint = androidx.compose.material3.MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        },
        colors = ListItemDefaults.listItemColors(
            containerColor = androidx.compose.material3.MaterialTheme.colorScheme.surfaceVariant.copy(alpha = 0.3)
        )
    )
}