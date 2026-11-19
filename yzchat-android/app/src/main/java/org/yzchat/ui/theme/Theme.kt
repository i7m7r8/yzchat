package org.yzchat.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

// Telegram X inspired color palette
private val DarkColors = darkColorScheme(
    primary = Color(0xFF0088CC),      // Telegram Blue
    primaryContainer = Color(0xFF006699),
    secondary = Color(0xFF2AABE2),    // Light Blue
    secondaryContainer = Color(0xFF1E88C7),
    tertiary = Color(0xFF50E3C2),     // Teal accent
    tertiaryContainer = Color(0xFF3CC9B0),
    surface = Color(0xFF1C1C1E),      // iOS-like dark surface
    surfaceVariant = Color(0xFF2C2C2E),
    background = Color(0xFF000000),   // True black for OLED
    error = Color(0xFFFF453A),
    onPrimary = Color.White,
    onSecondary = Color.White,
    onTertiary = Color.Black,
    onSurface = Color.White,
    onSurfaceVariant = Color(0xFFEBEBF5).copy(alpha = 0.6),
    outline = Color(0xFF3A3A3C),
    outlineVariant = Color(0xFF48484A),
    shadow = Color.Black,
    scrim = Color.Black,
    inverseSurface = Color(0xFFF2F2F7),
    inversePrimary = Color(0xFF007AFF),
)

private val LightColors = lightColorScheme(
    primary = Color(0xFF007AFF),
    primaryContainer = Color(0xFFD1E8FF),
    secondary = Color(0xFF007AFF),
    secondaryContainer = Color(0xFFD1E8FF),
    tertiary = Color(0xFF30B0A0),
    tertiaryContainer = Color(0xFFD0E8E4),
    surface = Color(0xFFFFFFFF),
    surfaceVariant = Color(0xFFF2F2F7),
    background = Color(0xFFFFFFFF),
    error = Color(0xFFFF3B30),
    onPrimary = Color.White,
    onSecondary = Color.White,
    onTertiary = Color.White,
    onSurface = Color.Black,
    onSurfaceVariant = Color(0xFF1C1C1E).copy(alpha = 0.6),
    outline = Color(0xFFC6C6C8),
    outlineVariant = Color(0xFFD1D1D6),
    shadow = Color.Black,
    scrim = Color.Black,
    inverseSurface = Color(0xFF1C1C1E),
    inversePrimary = Color(0xFF0088CC),
)

@Composable
fun YzchatTheme(
    darkTheme: Boolean = true, // Default to dark like Telegram X
    content: @Composable () -> Unit
) {
    val colorScheme = if (darkTheme) DarkColors else LightColors
    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        shapes = Shapes,
        content = content
    )
}

// Telegram X inspired typography
object Typography {
    val displayLarge = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W700,
        fontSize = 34.sp,
        lineHeight = 41.sp,
        letterSpacing = -0.5.sp
    )
    val displayMedium = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W700,
        fontSize = 28.sp,
        lineHeight = 36.sp,
        letterSpacing = 0.sp
    )
    val displaySmall = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W600,
        fontSize = 22.sp,
        lineHeight = 28.sp,
        letterSpacing = 0.sp
    )
    val headlineLarge = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W600,
        fontSize = 20.sp,
        lineHeight = 28.sp,
        letterSpacing = 0.sp
    )
    val headlineMedium = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W600,
        fontSize = 18.sp,
        lineHeight = 24.sp,
        letterSpacing = 0.sp
    )
    val headlineSmall = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W600,
        fontSize = 16.sp,
        lineHeight = 22.sp,
        letterSpacing = 0.sp
    )
    val titleLarge = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W600,
        fontSize = 16.sp,
        lineHeight = 22.sp,
        letterSpacing = 0.15.sp
    )
    val titleMedium = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W500,
        fontSize = 14.sp,
        lineHeight = 20.sp,
        letterSpacing = 0.1.sp
    )
    val titleSmall = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W500,
        fontSize = 12.sp,
        lineHeight = 16.sp,
        letterSpacing = 0.1.sp
    )
    val bodyLarge = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W400,
        fontSize = 16.sp,
        lineHeight = 24.sp,
        letterSpacing = 0.5.sp
    )
    val bodyMedium = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W400,
        fontSize = 14.sp,
        lineHeight = 20.sp,
        letterSpacing = 0.25.sp
    )
    val bodySmall = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W400,
        fontSize = 12.sp,
        lineHeight = 16.sp,
        letterSpacing = 0.4.sp
    )
    val labelLarge = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W500,
        fontSize = 14.sp,
        lineHeight = 20.sp,
        letterSpacing = 0.1.sp
    )
    val labelMedium = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W500,
        fontSize = 12.sp,
        lineHeight = 16.sp,
        letterSpacing = 0.5.sp
    )
    val labelSmall = androidx.compose.material3.TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = androidx.compose.ui.text.font.FontWeight.W500,
        fontSize = 11.sp,
        lineHeight = 16.sp,
        letterSpacing = 0.5.sp
    )
}

// Telegram X inspired shapes
object Shapes {
    val extraSmall = androidx.compose.material3.RoundedCornerShape(4.dp)
    val small = androidx.compose.material3.RoundedCornerShape(8.dp)
    val medium = androidx.compose.material3.RoundedCornerShape(12.dp)
    val large = androidx.compose.material3.RoundedCornerShape(16.dp)
    val extraLarge = androidx.compose.material3.RoundedCornerShape(24.dp)
    val full = androidx.compose.material3.RoundedCornerShape(9999.dp)
}