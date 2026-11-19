package org.yzchat.data.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import kotlinx.coroutines.flow.Flow
import org.yzchat.data.entity.MessageEntity

@Dao
interface MessageDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(message: MessageEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertAll(messages: List<MessageEntity>)

    @Update
    suspend fun update(message: MessageEntity)

    @Query("DELETE FROM messages WHERE id = :id")
    suspend fun delete(id: String)

    @Query("SELECT * FROM messages WHERE id = :id")
    suspend fun get(id: String): MessageEntity?

    @Query("SELECT * FROM messages WHERE conversationId = :conversationId ORDER BY timestamp DESC LIMIT :limit OFFSET :offset")
    suspend fun getByConversation(conversationId: String, limit: Int, offset: Int): List<MessageEntity>

    @Query("SELECT * FROM messages WHERE conversationId = :conversationId ORDER BY timestamp DESC LIMIT :limit")
    fun getByConversationFlow(conversationId: String, limit: Int): Flow<List<MessageEntity>>

    @Query("SELECT * FROM messages WHERE conversationId = :conversationId AND timestamp > :after ORDER BY timestamp DESC")
    suspend fun getNewerThan(conversationId: String, after: String): List<MessageEntity>

    @Query("SELECT * FROM messages WHERE conversationId = :conversationId AND isRead = 0 AND isOutgoing = 0")
    suspend fun getUnread(conversationId: String): List<MessageEntity>

    @Query("UPDATE messages SET isRead = 1 WHERE conversationId = :conversationId AND isOutgoing = 0")
    suspend fun markAllRead(conversationId: String)

    @Query("UPDATE messages SET status = :status WHERE id = :id")
    suspend fun updateStatus(id: String, status: String)

    @Query("SELECT COUNT(*) FROM messages WHERE conversationId = :conversationId AND isRead = 0 AND isOutgoing = 0")
    suspend fun countUnread(conversationId: String): Int

    @Query("SELECT * FROM messages WHERE status = 'Sending' OR status = 'Failed' ORDER BY timestamp ASC")
    suspend fun getPendingMessages(): List<MessageEntity>
}