package org.yzchat.data.repository

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import org.yzchat.data.dao.ContactDao
import org.yzchat.data.entity.ContactEntity
import org.yzchat.ffi.Contact
import org.yzchat.ffi.IdentityPublicKey
import org.yzchat.ffi.PeerCapabilities
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ContactRepository @Inject constructor(
    private val contactDao: ContactDao
) {

    suspend fun addContact(identity: IdentityPublicKey, alias: String) {
        val entity = ContactEntity(
            identityKey = identity.toHexString(),
            alias = alias,
            isOnline = false,
            capabilities = PeerCapabilities(
                supportsVoice = true,
                supportsVideo = true,
                supportsScreenShare = true,
                supportsFileTransfer = true,
                supportsGroups = true,
                maxVideoResolution = "P4K",
                supportedCodecs = listOf("AV1", "VP9", "OPUS")
            ).toJsonString()
        )
        contactDao.insert(entity)
    }

    suspend fun removeContact(identity: IdentityPublicKey) {
        contactDao.delete(identity.toHexString())
    }

    fun getContacts(): Flow<List<Contact>> = contactDao.getAll().map { entities ->
        entities.map { it.toContact() }
    }

    fun getOnlineContacts(): Flow<List<Contact>> = contactDao.getOnlineContacts().map { entities ->
        entities.map { it.toContact() }
    }

    fun searchContacts(query: String): Flow<List<Contact>> = contactDao.search("%$query%").map { entities ->
        entities.map { it.toContact() }
    }

    suspend fun updateContactAlias(identity: IdentityPublicKey, alias: String) {
        contactDao.get(identity.toHexString())?.let { entity ->
            contactDao.update(entity.copy(alias = alias))
        }
    }

    suspend fun updateOnlineStatus(identity: IdentityPublicKey, isOnline: Boolean) {
        contactDao.get(identity.toHexString())?.let { entity ->
            contactDao.update(entity.copy(isOnline = isOnline))
        }
    }
}

private fun ContactEntity.toContact(): Contact {
    return Contact(
        identity = IdentityPublicKey.fromHexString(identityKey),
        alias = alias,
        isOnline = isOnline,
        lastSeen = lastSeen?.toInstant().epochMilliseconds,
        capabilities = capabilities?.let { PeerCapabilities.fromJsonString(it) } ?: PeerCapabilities(
            supportsVoice = true,
            supportsVideo = true,
            supportsScreenShare = true,
            supportsFileTransfer = true,
            supportsGroups = true,
            maxVideoResolution = "P4K",
            supportedCodecs = listOf("AV1", "VP9", "OPUS")
        )
    )
}

private fun IdentityPublicKey.toHexString(): String {
    return bytes.joinToString("") { "%02X".format(it) }
}

private fun IdentityPublicKey.fromHexString(hex: String): IdentityPublicKey {
    return IdentityPublicKey(hex.chunked(2).map { it.toInt(16).toByte() }.toByteArray())
}

private fun PeerCapabilities.toJsonString(): String {
    return kotlinx.serialization.json.Json.encodeToString(this)
}

private fun PeerCapabilities.fromJsonString(json: String): PeerCapabilities {
    return kotlinx.serialization.json.Json.decodeFromString(json)
}