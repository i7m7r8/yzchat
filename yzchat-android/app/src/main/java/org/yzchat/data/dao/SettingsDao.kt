package org.yzchat.data.dao

import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Update
import kotlinx.coroutines.flow.Flow
import org.yzchat.data.entity.KeyBundleEntity
import org.yzchat.data.entity.PreKeyEntity
import org.yzchat.data.entity.IdentityEntity
import org.yzchat.data.entity.EncryptionKeyPairEntity
import org.yzchat.data.entity.SettingsEntity

@Dao
interface SettingsDao {

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun setSetting(setting: SettingsEntity)

    @Query("SELECT * FROM settings WHERE key = :key")
    suspend fun getSetting(key: String): SettingsEntity?

    @Query("SELECT * FROM settings")
    fun getAllSettings(): Flow<List<SettingsEntity>>

    @Query("DELETE FROM settings WHERE key = :key")
    suspend fun deleteSetting(key: String)

    // Key Bundles
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertKeyBundle(bundle: KeyBundleEntity)

    @Query("SELECT * FROM key_bundles WHERE identityKey = :identityKey")
    suspend fun getKeyBundle(identityKey: String): KeyBundleEntity?

    @Query("DELETE FROM key_bundles WHERE identityKey = :identityKey")
    suspend fun deleteKeyBundle(identityKey: String)

    @Query("DELETE FROM key_bundles WHERE expiresAt IS NOT NULL AND expiresAt < :now")
    suspend fun deleteExpiredKeyBundles(now: String)

    // Pre-Keys
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertPreKey(key: PreKeyEntity)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun insertPreKeys(keys: List<PreKeyEntity>)

    @Query("SELECT * FROM pre_keys WHERE keyId = :keyId")
    suspend fun getPreKey(keyId: Long): PreKeyEntity?

    @Query("SELECT * FROM pre_keys WHERE isUsed = 0 ORDER BY keyId ASC LIMIT :count")
    suspend fun getUnusedPreKeys(count: Int): List<PreKeyEntity>

    @Query("UPDATE pre_keys SET isUsed = 1 WHERE keyId = :keyId")
    suspend fun markPreKeyUsed(keyId: Long)

    @Query("DELETE FROM pre_keys WHERE isUsed = 1")
    suspend fun deleteUsedPreKeys()

    @Query("SELECT COUNT(*) FROM pre_keys WHERE isUsed = 0")
    suspend fun countUnusedPreKeys(): Int

    // Identity
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun setIdentity(identity: IdentityEntity)

    @Query("SELECT * FROM identity WHERE id = 1")
    suspend fun getIdentity(): IdentityEntity?

    @Query("DELETE FROM identity WHERE id = 1")
    suspend fun deleteIdentity()

    // Encryption Key Pair
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun setEncryptionKeyPair(keyPair: EncryptionKeyPairEntity)

    @Query("SELECT * FROM encryption_key_pair WHERE id = 1")
    suspend fun getEncryptionKeyPair(): EncryptionKeyPairEntity?

    @Query("DELETE FROM encryption_key_pair WHERE id = 1")
    suspend fun deleteEncryptionKeyPair()
}