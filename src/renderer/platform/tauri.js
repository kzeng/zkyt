import { invoke } from '@tauri-apps/api/core'

const invokeDb = (store, action, data) => {
  return invoke('db_request', { store, action, data })
}

export default {
  getSystemLocale: () => invoke('get_system_locale'),
  isWaylandPlatform: async () => false,
  openInNewWindow: (path, query) => {
    const searchParams = new URLSearchParams(query ?? {})
    const search = searchParams.size > 0 ? `?${searchParams.toString()}` : ''
    window.location.hash = `${path}${search}`
  },
  enableProxy: (url) => invoke('enable_proxy', { url }),
  disableProxy: () => invoke('disable_proxy'),
  setInvidiousAuthorization: (authorization, url) => {
    return invoke('set_invidious_authorization', { authorization, url })
  },
  clearInvidiousAuthorization: () => {
    return invoke('set_invidious_authorization', { authorization: null, url: null })
  },
  startPowerSaveBlocker: () => invoke('start_power_save_blocker'),
  stopPowerSaveBlocker: () => invoke('stop_power_save_blocker'),
  getReplaceHttpCache: async () => false,
  toggleReplaceHttpCache: () => {},
  requestPiP: () => document.querySelector('video.player')?.ui.getControls().togglePiP(),
  requestFullscreen: () => document.querySelector('video.player')?.ui.getControls().toggleFullScreen(),
  playerCacheGet: (key) => invoke('player_cache_get', { key }),
  playerCacheSet: (key, value) => invoke('player_cache_set', { key, value }),
  generatePoToken: (videoId, context) => invoke('generate_po_token', { videoId, context }),
  chooseDefaultFolder: () => {},
  writeToDefaultFolder: async () => false,
  relaunch: () => invoke('relaunch_app'),
  openInExternalPlayer: (payload) => invoke('open_in_external_player', { payload }),
  handleOpenInExternalPlayerResult: () => {},
  httpGetJson: (url, authorization = null) => invoke('http_get_json', { url: url.toString(), authorization }),
  setZoomFactor: () => {},
  getNavigationHistory: async () => [],
  dbSettings: (action, data) => invokeDb('settings', action, data),
  dbHistory: (action, data) => invokeDb('history', action, data),
  dbProfiles: (action, data) => invokeDb('profiles', action, data),
  dbPlaylists: (action, data) => invokeDb('playlists', action, data),
  dbSearchHistory: (action, data) => invokeDb('search-history', action, data),
  dbSubscriptionCache: (action, data) => invokeDb('subscription-cache', action, data),
  handleChangeView: () => {},
  handleOpenUrl: () => {},
  handleUpdateSearchInputText: () => {},
  handleSyncSettings: () => {},
  handleSyncHistory: () => {},
  handleSyncSearchHistory: () => {},
  handleSyncProfiles: () => {},
  handleSyncPlaylists: () => {},
  handleSyncSubscriptionCache: () => {},
}
