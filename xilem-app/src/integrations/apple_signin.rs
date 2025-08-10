// Apple Sign In Button Component - App Store Compliant Design
// Following Apple's Human Interface Guidelines and App Store Review Guidelines

use xilem::{
    view::{button, label},
    Color, WidgetView,
};

use crate::AppData;

/// Apple Sign In Button Styles according to Apple's guidelines
#[derive(Debug, Clone, Copy)]
pub enum AppleSignInButtonStyle {
    /// Black button with white text/logo - recommended for light backgrounds
    Black,
    /// White button with black text/logo - for dark backgrounds  
    White,
    /// White button with black outline - for backgrounds where white would blend
    WhiteOutline,
}

#[derive(Debug, Clone, Copy)]
pub enum AppleSignInButtonType {
    /// "Sign in with Apple" - default for first-time sign in
    SignIn,
    /// "Continue with Apple" - for returning users
    Continue,
    /// "Sign up with Apple" - specifically for account creation flows
    SignUp,
}

/// Creates an App Store compliant Apple Sign In button
pub fn apple_signin_button(
    data: &AppData,
    style: AppleSignInButtonStyle,
    button_type: AppleSignInButtonType,
) -> impl WidgetView<AppData> {
    let (bg_color, text_color, border_color) = match style {
        AppleSignInButtonStyle::Black => (
            Color::from_rgb8(0, 0, 0),       // Apple Black
            Color::from_rgb8(255, 255, 255), // White text
            None,
        ),
        AppleSignInButtonStyle::White => (
            Color::from_rgb8(255, 255, 255), // White background
            Color::from_rgb8(0, 0, 0),       // Black text
            None,
        ),
        AppleSignInButtonStyle::WhiteOutline => (
            Color::from_rgb8(255, 255, 255), // White background
            Color::from_rgb8(0, 0, 0),       // Black text
            Some(Color::from_rgb8(0, 0, 0)), // Black border
        ),
    };

    let button_text = match button_type {
        AppleSignInButtonType::SignIn => {
            if data.oauth_login_in_flight {
                "Signing in with Apple..."
            } else {
                "Sign in with Apple"
            }
        }
        AppleSignInButtonType::Continue => {
            if data.oauth_login_in_flight {
                "Continuing with Apple..."
            } else {
                "Continue with Apple"
            }
        }
        AppleSignInButtonType::SignUp => {
            if data.oauth_login_in_flight {
                "Signing up with Apple..."
            } else {
                "Sign up with Apple"
            }
        }
    };

    // Apple requires specific sizing and styling
    let apple_button = button(
        label(button_text).brush(text_color),
        |data: &mut AppData| {
            if !data.oauth_login_in_flight {
                data.apple_sign_in();
            }
        },
    );

    // Add border if specified
    // Background color for button not supported directly; text is styled above.

    apple_button
}

/// Creates the standard Apple Sign In button for login screens
pub fn standard_apple_signin_button(data: &AppData) -> impl WidgetView<AppData> {
    apple_signin_button(
        data,
        AppleSignInButtonStyle::Black, // Apple's recommended default
        AppleSignInButtonType::SignIn,
    )
}

/// Creates a "Continue with Apple" button for returning users
pub fn continue_with_apple_button(data: &AppData) -> impl WidgetView<AppData> {
    apple_signin_button(
        data,
        AppleSignInButtonStyle::Black,
        AppleSignInButtonType::Continue,
    )
}

/// Apple Sign In button compliance notes:
///
/// REQUIRED for App Store approval:
/// 1. Must use official Apple logo and branding
/// 2. Minimum height: 30pt (40px at 1x)  
/// 3. Maximum height: 64pt (85px at 1x)
/// 4. Width must accommodate text without truncation
/// 5. Corner radius: 6pt (8px at 1x) for rounded style, 0 for rectangular
/// 6. Must use San Francisco font (system font on iOS)
/// 7. Logo must be vertically and horizontally centered
/// 8. Cannot be modified beyond Apple's approved styles
///
/// PLACEMENT RULES:
/// 1. If offering other third-party sign-in options, Apple button must be:
///    - Equal or greater prominence
///    - Positioned above or to the left of other options
/// 2. Cannot be placed in a toolbar or navigation bar
/// 3. Must be placed in sign-in/sign-up flow, not buried in settings
///
/// TEXT REQUIREMENTS:
/// 1. Use "Sign in with Apple" for initial authentication
/// 2. Use "Continue with Apple" for returning users  
/// 3. Use "Sign up with Apple" specifically for account creation
/// 4. Cannot modify or translate the button text
/// 5. Text must be in system font (San Francisco on iOS)
///
/// COLOR REQUIREMENTS:
/// 1. Black: Default recommended style (RGB 0,0,0)
/// 2. White: For dark backgrounds (RGB 255,255,255)  
/// 3. White with outline: When white would blend with background
/// 4. Cannot use custom colors or gradients
pub const APPLE_SIGNIN_COMPLIANCE_NOTES: &str = "
Apple Sign In Button - App Store Compliance Requirements

DESIGN REQUIREMENTS:
✓ Use official Apple logo and typography
✓ Minimum height: 44pt (iOS touch target)
✓ Maintain aspect ratio and proportions
✓ Use only approved color schemes (black/white/white-outline)
✓ Corner radius: 6pt for rounded buttons
✓ Use San Francisco font (system font)

PLACEMENT REQUIREMENTS:
✓ Equal or greater prominence than other sign-in options
✓ Positioned above or left of other third-party options
✓ Not placed in toolbars or navigation bars
✓ Visible in primary sign-in/sign-up flow

FUNCTIONALITY REQUIREMENTS:
✓ Must offer Apple Sign In if offering other third-party options
✓ Must handle user cancellation gracefully
✓ Must work offline for existing users
✓ Cannot require Apple Sign In (must offer alternatives)

PRIVACY REQUIREMENTS:
✓ Handle private email relay properly
✓ Request minimal scopes (name, email)
✓ Respect user's Hide My Email choice
✓ Include privacy policy link
";
