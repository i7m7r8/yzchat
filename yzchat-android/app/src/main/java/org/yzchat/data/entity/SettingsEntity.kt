package org.yzchat.data.entity

import androidx.room.Entity
import androidx.room.PrimaryKey
import kotlinx.datetime.LocalDateTime

@Entity(tableName = "settings")
data class SettingsEntity(
    @PrimaryKey
    val key: String,
    val value: String,
    val updatedAt: LocalDateTime = LocalDateTime.now()
)

@Entity(tableName = "key_bundles", primaryKeys = ["identityKey"])
data class KeyBundleEntity(
    @PrimaryKey
    val identityKey: String, // Hex-encoded IdentityPublicKey bytes
    val bundleJson: String, // JSON-serialized KeyBundle
    val fetchedAt: LocalDateTime = LocalDateTime.now(),
    val expiresAt: LocalDateTime? = null
)

@Entity(tableName = "pre_keys", primaryKeys = ["keyId"])
data class PreKeyEntity(
    @PrimaryKey(autoGenerate = false)
    val keyId: Long,
    val keyJson: String, // JSON-serialized PreKey
    val isUsed: Boolean = false,
    val createdAt: LocalDateTime = LocalDateTime.now()
)

@Entity(tableName = "identity")
data class IdentityEntity(
    @PrimaryKey
    val id: Int = 1, // Singleton
    val secretKeyJson: String, // JSON-serialized IdentitySecretKey
    val createdAt: LocalDateTime = LocalDateTime.now()
)

@Entity(tableName = "encryption_key_pair")
data class EncryptionKeyPairEntity(
    @PrimaryKey
    val id: Int = 1, // Singleton
    val keypairJson: String, // JSON-serialized HybridKemKeypair (secret part)
    val createdAt: LocalDateTime = LocalDateTime.now()
)