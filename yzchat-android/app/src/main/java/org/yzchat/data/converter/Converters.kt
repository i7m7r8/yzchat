package org.yzchat.data.converter

import androidx.room.TypeConverter
import kotlinx.datetime.LocalDateTime
import kotlinx.datetime.TimeZone
import kotlinx.datetime.format.DateTimeFormatter
import kotlinx.serialization.json.Json
import kotlinx.serialization.decodeFromString
import kotlinx.serialization.encodeToString
import org.yzchat.data.entity.MessageType
import org.yzchat.data.entity.MessageStatus
import org.yzchat.data.entity.CallType
import org.yzchat.data.entity.CallState
import org.yzchat.data.entity.GroupRole
import org.yzchat.ffi.IdentityPublicKey
import org.yzchat.ffi.EncryptionPublicKey
import org.yzchat.ffi.PreKey
import org.yzchat.ffi.SignedPreKey
import org.yzchat.ffi.KeyBundle
import org.yzchat.ffi.PeerCapabilities

class Converters {

    @TypeConverter
    fun fromLocalDateTime(dateTime: LocalDateTime?): Long? {
        return dateTime?.toInstant(TimeZone.UTC)?.epochMilliseconds
    }

    @TypeConverter
    fun toLocalDateTime(millis: Long?): LocalDateTime? {
        return millis?.let { LocalDateTime.ofInstant(java.time.Instant.ofEpochMilli(it), TimeZone.UTC) }
    }

    @TypeConverter
    fun fromMessageType(type: MessageType?): String? = type?.name

    @TypeConverter
    fun toMessageType(name: String?): MessageType? = name?.let { MessageType.valueOf(it) }

    @TypeConverter
    fun fromMessageStatus(status: MessageStatus?): String? = status?.name

    @TypeConverter
    fun toMessageStatus(name: String?): MessageStatus? = name?.let { MessageStatus.valueOf(it) }

    @TypeConverter
    fun fromCallType(type: CallType?): String? = type?.name

    @TypeConverter
    fun toCallType(name: String?): CallType? = name?.let { CallType.valueOf(it) }

    @TypeConverter
    fun fromCallState(state: CallState?): String? = state?.name

    @TypeConverter
    fun toCallState(name: String?): CallState? = name?.let { CallState.valueOf(it) }

    @TypeConverter
    fun fromGroupRole(role: GroupRole?): String? = role?.name

    @TypeConverter
    fun toGroupRole(name: String?): GroupRole? = name?.let { GroupRole.valueOf(it) }

    @TypeConverter
    fun fromPeerCapabilities(caps: PeerCapabilities?): String? = caps?.let { Json.encodeToString(it) }

    @TypeConverter
    fun toPeerCapabilities(json: String?): PeerCapabilities? = json?.let { Json.decodeFromString(it) }

    @TypeConverter
    fun fromIdentityPublicKey(key: IdentityPublicKey?): String? = key?.let { Json.encodeToString(it) }

    @TypeConverter
    fun toIdentityPublicKey(json: String?): IdentityPublicKey? = json?.let { Json.decodeFromString(it) }

    @TypeConverter
    fun fromEncryptionPublicKey(key: EncryptionPublicKey?): String? = key?.let { Json.encodeToString(it) }

    @TypeConverter
    fun toEncryptionPublicKey(json: String?): EncryptionPublicKey? = json?.let { Json.decodeFromString(it) }

    @TypeConverter
    fun fromPreKey(key: PreKey?): String? = key?.let { Json.encodeToString(it) }

    @TypeConverter
    fun toPreKey(json: String?): PreKey? = json?.let { Json.decodeFromString(it) }

    @TypeConverter
    fun fromSignedPreKey(key: SignedPreKey?): String? = key?.let { Json.encodeToString(it) }

    @TypeConverter
    fun toSignedPreKey(json: String?): SignedPreKey? = json?.let { Json.decodeFromString(it) }

    @TypeConverter
    fun fromKeyBundle(bundle: KeyBundle?): String? = bundle?.let { Json.encodeToString(it) }

    @TypeConverter
    fun toKeyBundle(json: String?): KeyBundle? = json?.let { Json.decodeFromString(it) }

    @TypeConverter
    fun fromByteArray(bytes: ByteArray?): String? = bytes?.let { it.toHexString() }

    @TypeConverter
    fun toByteArray(hex: String?): ByteArray? = hex?.let { it.hexToByteArray() }
}

private fun ByteArray.toHexString(): String {
    return joinToString("") { "%02X".format(it) }
}

private fun String.hexToByteArray(): ByteArray {
    return chunked(2).map { it.toInt(16).toByte() }.toByteArray()
}