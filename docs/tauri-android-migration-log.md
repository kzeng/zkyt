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

### Current Limitations

- Android project generation and release packaging are available under `src-tauri/gen/android`.
- The generated APK is unsigned and must be signed before distribution outside local testing.
- Tauri database storage currently uses JSON document files, not SQLite.
- Tauri Local API `generate_po_token` is still unsupported because the current implementation depends on Electron.
- Tauri proxy configuration is still unsupported.
- Android external player integration is still unsupported and must be rebuilt using Android intents.
- The renderer still uses the desktop layout and has not been redesigned for phone ergonomics.
- Tauri build currently reuses the existing renderer webpack pipeline with a Tauri-specific wrapper config.

### Next Steps

- Decide how to handle Android signing keys and release-channel metadata.
- Replace the JSON document store with SQLite once the Tauri runtime can launch reliably.
- Add a mobile-specific renderer build flag and progressively hide desktop-only settings on Android.
- Build the Android watch-page shell and verify WebView playback behavior with Shaka Player.
