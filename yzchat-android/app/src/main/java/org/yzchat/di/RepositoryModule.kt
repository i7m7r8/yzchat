package org.yzchat.di

import dagger.hilt.InstallIn
import dagger.hilt.android.components.ApplicationComponent
import dagger.Module
import dagger.Provides
import javax.inject.Singleton
import org.yzchat.data.repository.ContactRepository
import org.yzchat.data.repository.MessageRepository
import org.yzchat.data.repository.GroupRepository
import org.yzchat.data.repository.SettingsRepository

@Module
@InstallIn(ApplicationComponent::class)
object RepositoryModule {

    @Provides
    @Singleton
    fun provideContactRepository(): ContactRepository {
        return ContactRepository()
    }

    @Provides
    @Singleton
    fun provideMessageRepository(): MessageRepository {
        return MessageRepository()
    }

    @Provides
    @Singleton
    fun provideGroupRepository(): GroupRepository {
        return GroupRepository()
    }

    @Provides
    @Singleton
    fun provideSettingsRepository(): SettingsRepository {
        return SettingsRepository()
    }
}