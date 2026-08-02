import tauriPlatform from './tauri'

const unsupportedPlatformMethod = (methodName) => {
  return () => {
    throw new Error(`Platform method "${methodName}" is not available in this runtime`)
  }
}

const noop = () => {}

const createUnsupportedPlatform = () => ({
  getSystemLocale: unsupportedPlatformMethod('getSystemLocale'),
  isWaylandPlatform: async () => false,
  openInNewWindow: noop,
  enableProxy: noop,
  disableProxy: noop,
  setInvidiousAuthorization: noop,
  clearInvidiousAuthorization: noop,
  startPowerSaveBlocker: noop,
  stopPowerSaveBlocker: noop,
  getReplaceHttpCache: async () => false,
  toggleReplaceHttpCache: noop,
  requestPiP: noop,
  requestFullscreen: noop,
  playerCacheGet: async () => undefined,
  playerCacheSet: async () => {},
  generatePoToken: unsupportedPlatformMethod('generatePoToken'),
  chooseDefaultFolder: noop,
  writeToDefaultFolder: async () => false,
  relaunch: noop,
  openInExternalPlayer: noop,
  handleOpenInExternalPlayerResult: noop,
  httpRequest: unsupportedPlatformMethod('httpRequest'),
  httpGetJson: unsupportedPlatformMethod('httpGetJson'),
  setZoomFactor: noop,
  getNavigationHistory: async () => [],
  dbSettings: unsupportedPlatformMethod('dbSettings'),
  dbHistory: unsupportedPlatformMethod('dbHistory'),
  dbProfiles: unsupportedPlatformMethod('dbProfiles'),
  dbPlaylists: unsupportedPlatformMethod('dbPlaylists'),
  dbSearchHistory: unsupportedPlatformMethod('dbSearchHistory'),
  dbSubscriptionCache: unsupportedPlatformMethod('dbSubscriptionCache'),
  handleChangeView: noop,
  handleOpenUrl: noop,
  handleUpdateSearchInputText: noop,
  handleSyncSettings: noop,
  handleSyncHistory: noop,
  handleSyncSearchHistory: noop,
  handleSyncProfiles: noop,
  handleSyncPlaylists: noop,
  handleSyncSubscriptionCache: noop,
})

const getRuntimePlatform = () => {
  if (globalThis.__TAURI_INTERNALS__ || globalThis.__TAURI__) {
    return tauriPlatform
  }

  if (globalThis.ftTauri) {
    return globalThis.ftTauri
  }

  if (globalThis.ftElectron) {
    return globalThis.ftElectron
  }

  return createUnsupportedPlatform()
}

const platform = getRuntimePlatform()

export default platform
