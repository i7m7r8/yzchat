package org.yzchat.data

import android.content.Context
import androidx.room.Database
import androidx.room.RoomDatabase
import androidx.room.TypeConverters
import org.yzchat.data.dao.*
import org.yzchat.data.entity.*
import org.yzchat.data.converter.Converters

@Database(
    entities = [
        ContactEntity::class,
        ConversationEntity::class,
        MessageEntity::class,
        GroupEntity::class,
        GroupMemberEntity::class,
        SettingsEntity::class,
        KeyBundleEntity::class,
        PreKeyEntity::class,
        IdentityEntity::class,
        EncryptionKeyPairEntity::class,
    ],
    version = 1,
    exportSchema = false
)
@TypeConverters(Converters::class)
abstract class YzchatDatabase : RoomDatabase() {

    abstract fun contactDao(): ContactDao
    abstract fun conversationDao(): ConversationDao
    abstract fun messageDao(): MessageDao
    abstract fun groupDao(): GroupDao
    abstract fun groupMemberDao(): GroupMemberDao
    abstract fun settingsDao(): SettingsDao
    abstract fun keyBundleDao(): KeyBundleDao
    abstract fun preKeyDao(): PreKeyDao
    abstract fun identityDao(): IdentityDao
    abstract fun encryptionKeyPairDao(): EncryptionKeyPairDao

    companion object {
        @Suppress("UNUSED_PARAMETER")
        @Volatile private var INSTANCE: YzchatDatabase? = null

        fun getDatabase(context: Context): YzchatDatabase {
            return INSTANCE ?: synchronized(this) {
                val instance = Room.databaseBuilder(
                    context.applicationContext,
                    YzchatDatabase::class.java,
                    "yzchat.db"
                ).fallbackToDestructiveMigration().build()
                INSTANCE = instance
                instance
            }
        }
    }
}