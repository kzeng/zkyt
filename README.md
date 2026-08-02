# ZKYT

ZKYT is a mobile-first Android adaptation of FreeTube built with Tauri. The project keeps FreeTube's privacy-oriented YouTube browsing model while replacing the Electron desktop shell with a lighter runtime suitable for Android phones.

## Mobile Focus

- Android app package: `com.kzeng.zkyt`
- App name: `ZKYT`
- Runtime: Tauri v2 + Android WebView
- Target device class: phones first, with special attention to Pixel-class portrait and landscape layouts
- Primary navigation: bottom tab bar optimized for touch

## Current Mobile Features

- Browse subscriptions, subscribed channels, popular videos, playlists, and settings from a phone-friendly bottom navigation bar.
- Search from the top app bar with mobile-safe spacing below the Android status bar.
- Use native Tauri/Rust HTTP requests for Invidious API calls to avoid WebView CORS failures.
- Keep Invidious as the preferred Android data backend while Local API support remains available for targeted fallback work.
- Store user settings, history, profiles, playlists, search history, and subscription cache through Tauri-backed local JSON stores.
- Build installable Android debug APKs for arm64 devices.

## Mobile UI Changes

- The top app bar avoids the Android system status bar.
- Desktop-style back/forward controls are hidden on mobile.
- Settings is a first-level mobile tab instead of being buried behind a desktop-style More menu.
- Subscription tabs stay on one row and scroll horizontally on narrow screens.
- The unstable Trending page is removed from navigation and `/trending` redirects to Most Popular.

## Android Build

Set up the Android environment, then build the debug APK:

```bash
. scripts/android-env.sh
corepack pnpm run tauri:android:build:debug
```

Install on a connected device:

```bash
. scripts/android-env.sh
adb install -r src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk
```

## Current Limitations

- Release APK signing is not configured.
- Some desktop-only FreeTube integrations are not available on Android.
- Local API playback support is incomplete where Electron-specific player URL transformation is still required.
- Mobile page layouts beyond the main shell still need more phone-first refinement.
