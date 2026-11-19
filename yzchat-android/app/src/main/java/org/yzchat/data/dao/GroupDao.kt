package org.yzchat.data.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import kotlinx.coroutines.flow.Flow
import org.yzchat.data.entity.GroupEntity
import org.yzchat.data.entity.GroupMemberEntity

@Dao
interface GroupDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insert(group: GroupEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertAll(groups: List<GroupEntity>)

    @Update
    suspend fun update(group: GroupEntity)

    @Query("DELETE FROM groups WHERE id = :id")
    suspend fun delete(id: String)

    @Query("SELECT * FROM groups WHERE id = :id")
    suspend fun get(id: String): GroupEntity?

    @Query("SELECT * FROM groups ORDER BY updatedAt DESC")
    fun getAll(): Flow<List<GroupEntity>>

    @Query("SELECT * FROM groups WHERE creatorKey = :creatorKey ORDER BY updatedAt DESC")
    fun getOwned(creatorKey: String): Flow<List<GroupEntity>>

    @Query("SELECT * FROM groups WHERE isPublic = 1 ORDER BY updatedAt DESC")
    fun getPublicGroups(): Flow<List<GroupEntity>>

    // Group Members
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertMember(member: GroupMemberEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertMembers(members: List<GroupMemberEntity>)

    @Query("DELETE FROM group_members WHERE groupId = :groupId AND identityKey = :identityKey")
    suspend fun removeMember(groupId: String, identityKey: String)

    @Query("SELECT * FROM group_members WHERE groupId = :groupId")
    suspend fun getMembers(groupId: String): List<GroupMemberEntity>

    @Query("SELECT * FROM group_members WHERE groupId = :groupId AND role IN ('Owner', 'Admin')")
    suspend fun getAdmins(groupId: String): List<GroupMemberEntity>

    @Query("SELECT * FROM group_members WHERE identityKey = :identityKey")
    suspend fun getGroupsForMember(identityKey: String): List<GroupMemberEntity>

    @Query("UPDATE group_members SET role = :role WHERE groupId = :groupId AND identityKey = :identityKey")
    suspend fun updateMemberRole(groupId: String, identityKey: String, role: String)
}