package org.yzchat.di

import android.content.Context
import androidx.room.Room
import dagger.hilt.InstallIn
import dagger.hilt.android.components.ApplicationComponent
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.Module
import dagger.Provides
import javax.inject.Singleton
import org.yzchat.data.YzchatDatabase

@Module
@InstallIn(ApplicationComponent::class)
object DatabaseModule {

    @Provides
    @Singleton
    fun provideDatabase(@ApplicationContext context: Context): YzchatDatabase {
        return Room.databaseBuilder(
            context,
            YzchatDatabase::class.java,
            "yzchat.db"
        ).fallbackToDestructiveMigration().build()
    }
}