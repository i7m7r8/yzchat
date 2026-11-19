package org.yzchat.data.repository

import kotlinx.coroutines.flow.Flow
import org.yzchat.data.dao.MessageDao
import org.yzchat.data.entity.MessageEntity
import org.yzchat.ffi.Message
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class MessageRepository @Inject constructor(
    private val messageDao: MessageDao
) {

    suspend fun saveMessage(message: Message) {
        val entity = message.toEntity()
        messageDao.insert(entity)
    }

    suspend fun saveMessages(messages: List<Message>) {
        val entities = messages.map { it.toEntity() }
        messageDao.insertAll(entities)
    }

    fun getMessages(conversationId: String, limit: Int = 50): Flow<List<Message>> =
        messageDao.getByConversationFlow(conversationId, limit).map { entities ->
            entities.map { it.toMessage() }
        }

    suspend fun loadMoreMessages(conversationId: String, beforeId: String, limit: Int): List<Message> {
        // Implementation would use pagination
        return emptyList()
    }

    suspend fun markAsRead(conversationId: String) {
        messageDao.markAllRead(conversationId)
    }

    suspend fun updateMessageStatus(messageId: String, status: String) {
        messageDao.updateStatus(messageId, status)
    }

    suspend fun getPendingMessages(): List<Message> {
        return messageDao.getPendingMessages().map { it.toMessage() }
    }
}

private fun Message.toEntity(): MessageEntity {
    return MessageEntity(
        id = id,
        conversationId = conversationId,
        senderKey = sender.toHexString(),
        content = content,
        messageType = when (messageType) {
            MessageType.Text -> MessageType.Text
            MessageType.Image -> MessageType.Image
            MessageType.Video -> MessageType.Video
            MessageType.File -> MessageType.File
            MessageType.Voice -> MessageType.Voice
            MessageType.Call -> MessageType.Call
            MessageType.GroupMessage -> MessageType.Text
            else -> MessageType.Text
        },
        timestamp = kotlinx.datetime.LocalDateTime.ofInstant(java.time.Instant.ofEpochMilli(timestamp)),
        isRead = isRead,
        isDelivered = isDelivered,
        isOutgoing = sender == getCurrentIdentity(), // Simplified
        status = when {
            isRead -> MessageStatus.Read
            isDelivered -> MessageStatus.Delivered
            else -> MessageStatus.Sent
        },
        attachments = attachments?.let { kotlinx.serialization.json.Json.encodeToString(it) },
        replyToId = null,
        forwardFromId = null
    )
}

private fun MessageEntity.toMessage(): Message {
    return Message(
        id = id,
        conversationId = conversationId,
        sender = IdentityPublicKey.fromHexString(senderKey),
        content = content,
        messageType = when (messageType) {
            MessageType.Text -> MessageType.Text
            MessageType.Image -> MessageType.Image
            MessageType.Video -> MessageType.Video
            MessageType.File -> MessageType.File
            MessageType.Voice -> MessageType.Voice
            MessageType.Call -> MessageType.Call
            else -> MessageType.Text
        },
        timestamp = timestamp.toInstant().epochMilliseconds,
        isRead = isRead,
        isDelivered = isDelivered,
        attachments = attachments?.let { kotlinx.serialization.json.Json.decodeFromString(it) } ?: emptyList()
    )
}

private fun IdentityPublicKey.toHexString(): String {
    return bytes.joinToString("") { "%02X".format(it) }
}

private fun IdentityPublicKey.fromHexString(hex: String): IdentityPublicKey {
    return IdentityPublicKey(hex.chunked(2).map { it.toInt(16).toByte() }.toByteArray())
}

private fun getCurrentIdentity(): String {
    // TODO: Get from storage
    return ""
}