package org.yzchat.data.repository

import kotlinx.coroutines.flow.Flow
import org.yzchat.data.dao.GroupDao
import org.yzchat.data.entity.GroupEntity
import org.yzchat.data.entity.GroupMemberEntity
import org.yzchat.ffi.Group
import org.yzchat.ffi.GroupMember
import org.yzchat.ffi.GroupRole
import org.yzchat.ffi.IdentityPublicKey
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class GroupRepository @Inject constructor(
    private val groupDao: GroupDao
) {

    suspend fun createGroup(group: Group) {
        val entity = group.toEntity()
        groupDao.insert(entity)
        groupDao.insertMembers(entity.members)
    }

    suspend fun joinGroup(groupId: String, member: GroupMember) {
        groupDao.insertMember(member.toEntity(groupId))
    }

    suspend fun leaveGroup(groupId: String, identity: IdentityPublicKey) {
        groupDao.removeMember(groupId, identity.toHexString())
    }

    fun getGroups(): Flow<List<Group>> = groupDao.getAll().map { entities ->
        entities.map { it.toGroup() }
    }

    fun getOwnedGroups(creatorKey: IdentityPublicKey): Flow<List<Group>> = groupDao.getOwned(creatorKey.toHexString()).map { entities ->
        entities.map { it.toGroup() }
    }

    suspend fun getGroupMembers(groupId: String): List<GroupMember> {
        return groupDao.getMembers(groupId).map { it.toGroupMember() }
    }

    suspend fun updateMemberRole(groupId: String, identity: IdentityPublicKey, role: GroupRole) {
        groupDao.updateMemberRole(groupId, identity.toHexString(), role.name)
    }
}

private fun GroupEntity.toGroup(): Group {
    return Group(
        id = id,
        name = name,
        creator = IdentityPublicKey.fromHexString(creatorKey),
        members = emptyList(), // Loaded separately
        createdAt = createdAt.toInstant().epochMilliseconds,
        updatedAt = updatedAt.toInstant().epochMilliseconds
    )
}

private fun GroupMemberEntity.toGroupMember(): GroupMember {
    return GroupMember(
        identity = IdentityPublicKey.fromHexString(identityKey),
        alias = alias,
        role = when (role) {
            GroupRole.Owner -> GroupRole.Owner
            GroupRole.Admin -> GroupRole.Admin
            GroupRole.Member -> GroupRole.Member
        },
        joinedAt = joinedAt.toInstant().epochMilliseconds
    )
}

private fun Group.toEntity(): GroupEntity {
    return GroupEntity(
        id = id,
        name = name,
        creatorKey = creator.toHexString(),
        createdAt = kotlinx.datetime.LocalDateTime.ofInstant(java.time.Instant.ofEpochMilli(createdAt)),
        updatedAt = kotlinx.datetime.LocalDateTime.ofInstant(java.time.Instant.ofEpochMilli(updatedAt)),
        maxMembers = 200000
    )
}

private fun GroupMember.toEntity(groupId: String): GroupMemberEntity {
    return GroupMemberEntity(
        groupId = groupId,
        identityKey = identity.toHexString(),
        alias = alias,
        role = when (role) {
            GroupRole.Owner -> GroupRole.Owner
            GroupRole.Admin -> GroupRole.Admin
            GroupRole.Member -> GroupRole.Member
        },
        joinedAt = kotlinx.datetime.LocalDateTime.ofInstant(java.time.Instant.ofEpochMilli(joinedAt))
    )
}

private fun IdentityPublicKey.toHexString(): String {
    return bytes.joinToString("") { "%02X".format(it) }
}

private fun IdentityPublicKey.fromHexString(hex: String): IdentityPublicKey {
    return IdentityPublicKey(hex.chunked(2).map { it.toInt(16).toByte() }.toByteArray())
}