package org.yzchat.data.entity

import androidx.room.Entity
import androidx.room.PrimaryKey
import kotlinx.datetime.LocalDateTime
import org.yzchat.ffi.IdentityPublicKey
import org.yzchat.ffi.PeerCapabilities

@Entity(tableName = "contacts", primaryKeys = ["identityKey"])
data class ContactEntity(
    @PrimaryKey
    val identityKey: String, // Hex-encoded IdentityPublicKey bytes
    val alias: String = "",
    val isOnline: Boolean = false,
    val lastSeen: LocalDateTime? = null,
    val capabilities: String? = null, // JSON-serialized PeerCapabilities
    val createdAt: LocalDateTime = LocalDateTime.now(),
    val updatedAt: LocalDateTime = LocalDateTime.now()
)