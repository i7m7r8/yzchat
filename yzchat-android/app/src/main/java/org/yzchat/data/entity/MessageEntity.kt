package org.yzchat.data.entity

import androidx.room.Entity
import androidx.room.ForeignKey
import androidx.room.Index
import androidx.room.PrimaryKey
import kotlinx.datetime.LocalDateTime
import org.yzchat.ffi.IdentityPublicKey

@Entity(
    tableName = "messages",
    primaryKeys = ["id"],
    foreignKeys = [
        ForeignKey(
            entity = ConversationEntity::class,
            parentColumns = ["id"],
            childColumns = ["conversationId"],
            onDelete = ForeignKey.CASCADE
        )
    ],
    indices = [
        Index("conversationId"),
        Index("timestamp")
    ]
)
data class MessageEntity(
    @PrimaryKey
    val id: String,
    val conversationId: String,
    val senderKey: String, // Hex-encoded IdentityPublicKey bytes
    val content: String, // Decrypted text or encrypted JSON
    val messageType: MessageType = MessageType.Text,
    val timestamp: LocalDateTime = LocalDateTime.now(),
    val isRead: Boolean = false,
    val isDelivered: Boolean = false,
    val isOutgoing: Boolean = false,
    val status: MessageStatus = MessageStatus.Sent,
    val attachments: String? = null, // JSON array of Attachment
    val replyToId: String? = null,
    val forwardFromId: String? = null
)

enum class MessageType {
    Text, Image, Video, File, Voice, Call, Location, Contact, Poll, Sticker
}

enum class MessageStatus {
    Sending, Sent, Delivered, Read, Failed
}