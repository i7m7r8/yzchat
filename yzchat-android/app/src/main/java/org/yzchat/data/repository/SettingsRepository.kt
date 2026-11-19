package org.yzchat.data.repository

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import org.yzchat.data.dao.SettingsDao
import org.yzchat.data.entity.EncryptionKeyPairEntity
import org.yzchat.data.entity.IdentityEntity
import org.yzchat.data.entity.KeyBundleEntity
import org.yzchat.data.entity.PreKeyEntity
import org.yzchat.data.entity.SettingsEntity
import org.yzchat.ffi.IdentityPublicKey
import org.yzchat.ffi.IdentitySecretKey
import org.yzchat.ffi.KeyBundle
import org.yzchat.ffi.PreKey
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class SettingsRepository @Inject constructor(
    private val settingsDao: SettingsDao
) {

    suspend fun setSetting(key: String, value: String) {
        settingsDao.setSetting(SettingsEntity(key = key, value = value))
    }

    suspend fun getSetting(key: String): String? {
        return settingsDao.getSetting(key)?.value
    }

    fun getAllSettings(): Flow<List<SettingsEntity>> = settingsDao.getAllSettings()

    suspend fun deleteSetting(key: String) {
        settingsDao.deleteSetting(key)
    }

    // Identity
    suspend fun saveIdentity(key: IdentitySecretKey) {
        val json = kotlinx.serialization.json.Json.encodeToString(key)
        settingsDao.setIdentity(IdentityEntity(secretKeyJson = json))
    }

    suspend fun loadIdentity(): IdentitySecretKey? {
        return settingsDao.getIdentity()?.let {
            kotlinx.serialization.json.Json.decodeFromString(it.secretKeyJson)
        }
    }

    suspend fun deleteIdentity() {
        settingsDao.deleteIdentity()
    }

    // Encryption Key Pair
    suspend fun saveEncryptionKeyPair(keyPairJson: String) {
        settingsDao.setEncryptionKeyPair(EncryptionKeyPairEntity(keypairJson = keyPairJson))
    }

    suspend fun loadEncryptionKeyPair(): String? {
        return settingsDao.getEncryptionKeyPair()?.keypairJson
    }

    suspend fun deleteEncryptionKeyPair() {
        settingsDao.deleteEncryptionKeyPair()
    }

    // Key Bundles
    suspend fun saveKeyBundle(identity: IdentityPublicKey, bundle: KeyBundle) {
        val json = kotlinx.serialization.json.Json.encodeToString(bundle)
        settingsDao.insertKeyBundle(KeyBundleEntity(
            identityKey = identity.toHexString(),
            bundleJson = json
        ))
    }

    suspend fun loadKeyBundle(identity: IdentityPublicKey): KeyBundle? {
        return settingsDao.getKeyBundle(identity.toHexString())?.let {
            kotlinx.serialization.json.Json.decodeFromString(it.bundleJson)
        }
    }

    suspend fun deleteKeyBundle(identity: IdentityPublicKey) {
        settingsDao.deleteKeyBundle(identity.toHexString())
    }

    suspend fun cleanupExpiredKeyBundles() {
        val now = kotlinx.datetime.LocalDateTime.now().toString()
        settingsDao.deleteExpiredKeyBundles(now)
    }

    // Pre-Keys
    suspend fun savePreKeys(keys: List<PreKey>) {
        val entities = keys.map { key ->
            PreKeyEntity(
                keyId = key.keyId.toLong(),
                keyJson = kotlinx.serialization.json.Json.encodeToString(key)
            )
        }
        settingsDao.insertPreKeys(entities)
    }

    suspend fun getUnusedPreKeys(count: Int): List<PreKey> {
        return settingsDao.getUnusedPreKeys(count).map { entity ->
            kotlinx.serialization.json.Json.decodeFromString(entity.keyJson)
        }
    }

    suspend fun consumePreKey(keyId: Int): PreKey? {
        return settingsDao.getPreKey(keyId.toLong())?.let { entity ->
            settingsDao.markPreKeyUsed(keyId.toLong())
            kotlinx.serialization.json.Json.decodeFromString(entity.keyJson)
        }
    }

    suspend fun deleteUsedPreKeys() {
        settingsDao.deleteUsedPreKeys()
    }

    fun countUnusedPreKeys(): Flow<Int> = settingsDao.countUnusedPreKeys().map { it }
}

private fun IdentityPublicKey.toHexString(): String {
    return bytes.joinToString("") { "%02X".format(it) }
}