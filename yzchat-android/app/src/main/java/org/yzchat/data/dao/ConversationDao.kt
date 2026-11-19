package org.yzchat.data.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import kotlinx.coroutines.flow.Flow
import org.yzchat.data.entity.ConversationEntity

@Dao
interface ConversationDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(conversation: ConversationEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertAll(conversations: List<ConversationEntity>)

    @Update
    suspend fun update(conversation: ConversationEntity)

    @Query("DELETE FROM conversations WHERE id = :id")
    suspend fun delete(id: String)

    @Query("SELECT * FROM conversations WHERE id = :id")
    suspend fun get(id: String): ConversationEntity?

    @Query("SELECT * FROM conversations WHERE isArchived = 0 ORDER BY updatedAt DESC")
    fun getAllActive(): Flow<List<ConversationEntity>>

    @Query("SELECT * FROM conversations WHERE isArchived = 1 ORDER BY updatedAt DESC")
    fun getArchived(): Flow<List<ConversationEntity>>

    @Query("SELECT * FROM conversations WHERE participantKeys LIKE :key ORDER BY updatedAt DESC")
    fun getByParticipant(key: String): Flow<List<ConversationEntity>>

    @Query("SELECT * FROM conversations WHERE isGroup = 1 AND isArchived = 0 ORDER BY updatedAt DESC")
    fun getGroups(): Flow<List<ConversationEntity>>

    @Query("UPDATE conversations SET unreadCount = unreadCount + 1 WHERE id = :id")
    suspend fun incrementUnread(id: String)

    @Query("UPDATE conversations SET unreadCount = 0 WHERE id = :id")
    suspend fun clearUnread(id: String)

    @Query("UPDATE conversations SET lastMessageId = :messageId, updatedAt = :updatedAt WHERE id = :id")
    suspend fun updateLastMessage(id: String, messageId: String, updatedAt: String)

    @Query("SELECT COUNT(*) FROM conversations WHERE isArchived = 0")
    suspend fun countActive(): Int
}