# Tauri Android Migration Log

## 2026-08-02

### Goal

Refactor FreeTube toward a Tauri v2 runtime so the project can eventually support Android mobile devices without Electron.

### Decisions

- Keep the existing Vue renderer for the first migration phase.
- Introduce a renderer platform abstraction before removing Electron code.
- Add Tauri as a parallel runtime instead of deleting Electron immediately.
- Start Tauri storage with JSON document files to match the current NeDB-shaped data model, then migrate the storage implementation to SQLite later without changing renderer call sites again.
- Leave Android-only integrations, such as external players, power management, downloads, and native picture-in-picture, behind explicit Tauri command boundaries.

### Implemented

- Added `src/renderer/platform/index.js` as the shared renderer platform entry.
- Added `src/renderer/platform/tauri.js` for Tauri command calls.
- Replaced direct renderer uses of `window.ftElectron` with `platform`.
- Added Tauri npm dependencies:
  - `@tauri-apps/api@2.11.1`
  - `@tauri-apps/cli@2.11.4`
- Added package scripts:
  - `dev:tauri`
  - `dev:tauri-renderer`
  - `pack:tauri`
  - `tauri`
  - `tauri:android:init`
  - `tauri:android:dev`
  - `tauri:android:build`
  - `tauri:android:build:debug`
- Added `src-tauri` with:
  - Tauri v2 config
  - Rust app entrypoints
  - default capability config
  - RGBA app icon converted from the existing FreeTube icon
  - initial Rust command bridge
- Implemented Tauri player cache get/set commands using app cache storage.
- Added a dedicated Tauri renderer webpack config that disables Electron-only code paths and Local API support for the initial Tauri runtime.
- Added a dedicated Tauri renderer dev server so `tauri dev` uses the Tauri platform adapter instead of the PWA datastore handler.
- Tauri settings reads now filter Electron-only settings and force the initial runtime to use Invidious while Local API support is unavailable.
- Implemented initial Tauri database command support for these stores:
  - `settings`
  - `history`
  - `profiles`
  - `playlists`
  - `search-history`
  - `subscription-cache`

### Verified

- `corepack pnpm run eslint-lint`
- `cargo check` in `src-tauri`
- `corepack pnpm run pack:renderer`
- `corepack pnpm run pack:tauri`
- `corepack pnpm tauri --version`
- `source scripts/android-env.sh && java -version`
- `source scripts/android-env.sh && sdkmanager --version`
- `source scripts/android-env.sh && adb version`
- `source scripts/android-env.sh && rustup target list --installed`
- `source scripts/android-env.sh && corepack pnpm tauri android init`
- `source scripts/android-env.sh && rustup component add rustfmt`
- `source scripts/android-env.sh && corepack pnpm tauri android build`
- `source scripts/android-env.sh && corepack pnpm tauri android build --debug --apk --target aarch64`
- `source scripts/android-env.sh && $ANDROID_HOME/build-tools/36.1.0/apksigner verify --verbose --print-certs src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`

### Android Environment Setup

- `sudo apt` was not usable in this session because it required interactive password authentication.
- Installed Android CLI with the official user-level installer:
  - `curl -fsSL https://dl.google.com/android/cli/latest/linux_x86_64/install.sh | bash`
- Android CLI installed `android` to:
  - `$HOME/.local/bin/android`
- Android CLI selected this SDK location:
  - `$HOME/Android/Sdk`
- Installed SDK packages:
  - `platform-tools` 37.0.1
  - `platforms/android-36` 2.0.0
  - `build-tools/36.1.0` 36.1.0
  - `cmdline-tools/latest` 22.0.0
  - `ndk/29.0.14206865`
  - `cmake/3.31.6`
- Android CLI bundled a JRE-like runtime, but it did not expose a standard `bin/java`, so it was not suitable for `JAVA_HOME`.
- Installed user-level Temurin JDK 17 from Adoptium:
  - `OpenJDK17U-jdk_x64_linux_hotspot_17.0.19_10.tar.gz`
  - Installed to `$HOME/.local/share/jdks/jdk-17.0.19+10`
- Added `scripts/android-env.sh` to configure:
  - `JAVA_HOME`
  - `ANDROID_HOME`
  - `ANDROID_SDK_ROOT`
  - `ANDROID_NDK_HOME`
  - `NDK_HOME`
  - `PATH`
- Installed user-level `rustup` because `tauri android init` needs `rustup target add` for Android Rust targets and the system Rust installation did not include `rustup`.
- `tauri android init` installed Rust Android targets:
  - `aarch64-linux-android`
  - `armv7-linux-androideabi`
  - `i686-linux-android`
  - `x86_64-linux-android`
- Generated the Tauri Android Gradle project under:
  - `src-tauri/gen/android`
- First `tauri android build` reached Gradle packaging but failed because the generated Gradle Rust task starts `pnpm` directly, while this environment only had `corepack pnpm`.
- Added `scripts/pnpm` as a project-local wrapper around `corepack pnpm` and placed `scripts/` at the front of `PATH` in `scripts/android-env.sh`.
- Patched the generated Android Gradle `BuildTask.kt` to invoke the project-local `scripts/pnpm` wrapper by absolute path, because Java/Gradle still failed to spawn `pnpm` through PATH lookup. The generated `rootDirRel` points at `src-tauri`, so the wrapper path is resolved as `rootDirRel/../scripts/pnpm`.
- The next Android build passed the wrapper issue and failed because the Rust library lacked Tauri mobile runtime symbols. Added `#[cfg_attr(mobile, tauri::mobile_entry_point)]` to the shared Rust `run` entrypoint.
- Installed the `rustfmt` component into the user-level Rust toolchain so Rust formatting checks can run with the same environment.
- The Android release build now succeeds and creates:
  - `src-tauri/gen/android/app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk`
  - `src-tauri/gen/android/app/build/outputs/bundle/universalRelease/app-universal-release.aab`
- Gradle currently reports deprecation warnings for future Gradle 9 compatibility, but they do not block the Android build.
- Expanded `.gitignore` coverage for Tauri/Rust targets, Android Gradle caches, generated packaged assets, generated native libraries, and Android signing files while keeping generated Android project source files trackable.
- A Pixel 9 Pro install attempt failed when using `app-universal-release-unsigned.apk`. `apksigner` confirmed that package does not verify and is missing `META-INF/MANIFEST.MF`, so Android correctly rejects it as unsigned.
- Added `tauri:android:build:debug` for an installable arm64 debug APK:
  - `source scripts/android-env.sh && corepack pnpm run tauri:android:build:debug`
- Generated and verified this debug APK:
  - `src-tauri/gen/android/app/build/outputs/apk/universal/debug/app-universal-debug.apk`
  - Signature verification passes with Android debug certificate and APK Signature Scheme v2.
  - `aapt` reports `native-code: 'arm64-v8a'`, which matches Pixel 9 Pro.
- `adb devices -l` detected the connected Pixel device and local install verification now passes.
- Renamed the Android/Tauri app identity for the ZKYT fork:
  - Tauri `productName`: `ZKYT`
  - Tauri/Android identifier: `com.kzeng.zkyt`
  - Android launcher labels: `ZKYT`
  - Android `MainActivity` package: `com.kzeng.zkyt`
- Fixed the first-launch Android black screen after the ZKYT rename:
  - WebView DevTools showed that the Vue app mounted to an empty comment node because JSON bootstrap requests returned the SPA fallback HTML.
  - Affected URLs included `/static/locales/*.json`, `/static/invidious-instances.json`, and `/static/geolocations/*.json`.
  - The Tauri renderer webpack config now disables locale JSON compression and copies the required static JSON files into `dist`.
  - A full `dist` and `src-tauri/target` cleanup was required so Tauri/Cargo embedded the corrected static assets into the Android library.
- Rebuilt and reinstalled the debug APK on the connected Pixel device after the asset fix.
- Verified the ZKYT Android app opens and renders the FreeTube UI instead of a black screen.
- Observed a separate runtime toast after launch:
  - `Invidious API错误（点击复制）: TypeError: Failed to fetch`
  - This is a network/API request issue after rendering succeeds, not the black screen root cause.
- Removed the FreeTube About page and all side navigation entries that linked to it for the ZKYT fork.
- Cleared `README.md`.
- Updated the bundled Invidious static instance list after API testing showed several old entries timing out, returning 401, or showing bot checks. The initial Tauri/Android instance list now starts with `https://inv.zoomerville.com`.
- Adjusted Tauri top navigation behavior:
  - Back and forward buttons no longer rely on Android WebView's Navigation API state.
  - The search icon now expands and focuses the search input on mobile.
- Fixed two follow-up Android issues:
  - Removed Android `enableEdgeToEdge()` from `MainActivity` because it placed the WebView top navigation under the system status bar, making the visible top icons hard or impossible to tap.
  - Tauri startup now clears a persisted default Invidious instance when it is no longer present in the bundled usable instance list, then falls back to the bundled instance.
- Added a Tauri-native HTTPS JSON fetch command and moved Invidious API JSON calls through it on Tauri/Android. This avoids Android WebView CORS failures such as `No 'Access-Control-Allow-Origin' header` on otherwise reachable Invidious API endpoints.
- Added Invidious native-fetch fallback across bundled instances on Tauri/Android and allowed non-CORS instances in the Tauri instance list because JSON API calls no longer run inside WebView.
- Expanded native HTTP error formatting so Android logs include the underlying DNS/TLS/timeout cause instead of only `error sending request`.
- Applied Android system bar insets to the Tauri WebView from `MainActivity` so the top navigation is laid out below the status bar.
- Started mobile-specific UI restructuring instead of continuing to reuse the desktop shell on phones:
  - Mobile top navigation now hides desktop-style back/forward controls and keeps a centered app identity with a dedicated search entry.
  - Mobile bottom navigation is now a fixed five-tab touch bar with larger hit targets.
  - Mobile content spacing now reserves explicit room for the top app bar and bottom tab bar.
  - The mobile More menu is presented above the bottom tab bar instead of as a narrow desktop side-menu remnant.
- Tightened the Android mobile shell after Pixel 9 Pro testing:
  - Added a Tauri-only mobile top safe-area offset so the logo, search icon, expanded search field, and page content start below the Android status bar instead of occupying the system icon area.
  - Made the mobile search icon an explicit right-side grid target with its own accessible label/title and kept focus behavior when the search field opens.
  - Replaced the mobile bottom navigation "More" tab with a direct Settings tab. The desktop sidebar keeps its more menu for secondary destinations, but phones now get settings as a first-level destination instead of a dropdown-style desktop pattern.
  - Raised the mobile shell breakpoint and matching search toggle logic to 960px so Pixel-class phones in landscape still use the mobile top/bottom navigation instead of falling back to the desktop sidebar layout.
- Investigated empty Popular/Search results on a Pixel using Clash VPN. The installed app has `INTERNET` granted and Android reports the VPN network as validated, so the issue is not a missing Android permission. Added Tauri/Invidious fallback for empty first-page `popular` and `search` arrays because some instances can return successful but empty API responses under specific instance/VPN/region combinations. Also fixed the search page to stop loading if an API returns no result object, and truncated native HTTP error bodies so Android toasts/logs remain readable.
- Re-tested the bundled Invidious instances from the development machine on 2026-08-02. `https://inv.nadeko.net/api/v1/popular` returned valid JSON, while `invidious.f5.si` and `yt.chocolatemoo53.com` returned 403 and `inv.zoomerville.com` hung during probing. The bundled list was reduced to `https://inv.nadeko.net` so Android starts from the currently verified API instance and clears stale persisted defaults that are no longer bundled.

### Current Limitations

- Android project generation and release packaging are available under `src-tauri/gen/android`.
- The generated APK is unsigned and must be signed before distribution outside local testing.
- For manual phone testing, use the debug APK rather than `app-universal-release-unsigned.apk`.
- Tauri database storage currently uses JSON document files, not SQLite.
- Tauri Local API `generate_po_token` is still unsupported because the current implementation depends on Electron.
- Tauri proxy configuration is still unsupported.
- Android external player integration is still unsupported and must be rebuilt using Android intents.
- The renderer still uses the desktop layout and has not been redesigned for phone ergonomics.
- Tauri build currently reuses the existing renderer webpack pipeline with a Tauri-specific wrapper config.

### Next Steps

- Decide how to handle Android signing keys and release-channel metadata.
- Verify the Android Invidious request path on-device after the instance list update.
- Replace the JSON document store with SQLite once the Tauri runtime can launch reliably.
- Add a mobile-specific renderer build flag and progressively hide desktop-only settings on Android.
- Build the Android watch-page shell and verify WebView playback behavior with Shaka Player.
