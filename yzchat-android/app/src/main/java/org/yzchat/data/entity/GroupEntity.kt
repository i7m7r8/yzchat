package org.yzchat.data.entity

import androidx.room.Entity
import androidx.room.PrimaryKey
import kotlinx.datetime.LocalDateTime
import org.yzchat.ffi.IdentityPublicKey

@Entity(tableName = "groups")
data class GroupEntity(
    @PrimaryKey
    val id: String,
    val name: String,
    val creatorKey: String, // Hex-encoded IdentityPublicKey bytes
    val description: String = "",
    val avatarHash: String? = null, // Blob hash for group avatar
    val createdAt: LocalDateTime = LocalDateTime.now(),
    val updatedAt: LocalDateTime = LocalDateTime.now(),
    val isPublic: Boolean = false,
    val maxMembers: Int = 200000
)

@Entity(tableName = "group_members", primaryKeys = ["groupId", "identityKey"])
data class GroupMemberEntity(
    val groupId: String,
    val identityKey: String, // Hex-encoded IdentityPublicKey bytes
    val alias: String = "",
    val role: GroupRole = GroupRole.Member,
    val joinedAt: LocalDateTime = LocalDateTime.now(),
    val invitedBy: String? = null
)

enum class GroupRole {
    Owner, Admin, Member
}