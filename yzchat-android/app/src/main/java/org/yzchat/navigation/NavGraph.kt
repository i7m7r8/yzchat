package org.yzchat.navigation

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavGraphBuilder
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import org.yzchat.ui.call.CallScreen
import org.yzchat.ui.chat.ChatScreen
import org.yzchat.ui.main.MainScreen
import org.yzchat.ui.settings.SettingsScreen
import org.yzchat.ui.onboarding.OnboardingScreen
import org.yzchat.ui.onboarding.WelcomeScreen

@Composable
fun AppNavHost() {
    val navController = rememberNavController()
    val yzchatCore = viewModel<YzchatViewModel>()

    androidx.navigation.compose.NavHost(navController, startDestination = "onboarding") {
        composable("onboarding") {
            OnboardingScreen(onFinish = { navController.navigate("welcome") })
        }

        composable("welcome") {
            WelcomeScreen(onGetStarted = { navController.navigate("main") })
        }

        composable("main") {
            MainScreen(
                onChatClick = { name -> navController.navigate("chat/$name") },
                onNewChatClick = { /* Show new chat dialog */ },
                onCallClick = { /* Show call log */ },
                onSettingsClick = { navController.navigate("settings") }
            )
        }

        composable(
            route = "chat/{contactName}",
            arguments = listOf(androidx.navigation.navArgument("contactName") { type = androidx.navigation.NavType.StringType })
        ) { backStackEntry ->
            val contactName = backStackEntry.getString() ?: ""
            ChatScreen(
                contactName = contactName,
                onBackClick = { navController.popBackStack() },
                onCallClick = { navController.navigate("call/$contactName/voice") },
                onVideoClick = { navController.navigate("call/$contactName/video") },
                onSendMessage = { text -> /* Send via yzchatCore */ }
            )
        }

        composable(
            route = "call/{contactName}/{callType}",
            arguments = listOf(
                androidx.navigation.navArgument("contactName") { type = androidx.navigation.NavType.StringType },
                androidx.navigation.navArgument("callType") { type = androidx.navigation.NavType.StringType }
            )
        ) { backStackEntry ->
            val contactName = backStackEntry.getString() ?: ""
            val callTypeStr = backStackEntry.getString() ?: "voice"
            val callType = if (callTypeStr == "video") CallType.Video else CallType.Voice
            CallScreen(
                contactName = contactName,
                callType = callType,
                isIncoming = false,
                onAccept = { /* Answer */ },
                onDecline = { navController.popBackStack() },
                onEndCall = { navController.popBackStack() },
                onMuteToggle = { /* Mute */ },
                onSpeakerToggle = { /* Speaker */ },
                onCameraToggle = { /* Camera */ },
                onFlipCamera = { /* Flip */ },
                onPipToggle = { /* PiP */ }
            )
        }

        composable("settings") {
            SettingsScreen(onBackClick = { navController.popBackStack() })
        }
    }
}

enum class CallType { Voice, Video }