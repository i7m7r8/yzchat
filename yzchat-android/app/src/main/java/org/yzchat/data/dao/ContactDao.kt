package org.yzchat.data.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import kotlinx.coroutines.flow.Flow
import org.yzchat.data.entity.ContactEntity

@Dao
interface ContactDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(contact: ContactEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertAll(contacts: List<ContactEntity>)

    @Update
    suspend fun update(contact: ContactEntity)

    @Query("DELETE FROM contacts WHERE identityKey = :identityKey")
    suspend fun delete(identityKey: String)

    @Query("SELECT * FROM contacts WHERE identityKey = :identityKey")
    suspend fun get(identityKey: String): ContactEntity?

    @Query("SELECT * FROM contacts ORDER BY updatedAt DESC")
    fun getAll(): Flow<List<ContactEntity>>

    @Query("SELECT * FROM contacts WHERE isOnline = 1 ORDER BY alias ASC")
    fun getOnlineContacts(): Flow<List<ContactEntity>>

    @Query("SELECT * FROM contacts WHERE alias LIKE :query OR identityKey LIKE :query ORDER BY updatedAt DESC")
    fun search(query: String): Flow<List<ContactEntity>>

    @Query("SELECT COUNT(*) FROM contacts")
    suspend fun count(): Int
}