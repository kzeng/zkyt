import { invoke } from '@tauri-apps/api/core'

const invokeDb = (store, action, data) => {
  return invoke('db_request', { store, action, data })
}

const androidHttpBridge = () => window.ZkytAndroidHttp

const androidHttpRequest = async (payload) => {
  const bridge = androidHttpBridge()

  if (!bridge) {
    return await invoke('http_request', { payload })
  }

  const response = JSON.parse(bridge.request(JSON.stringify(payload)))

  if (response.error) {
    throw new Error(response.error)
  }

  return response
}

const httpGetJson = async (url, authorization = null) => {
  const headers = {
    accept: 'application/json'
  }

  if (authorization) {
    headers.authorization = authorization
  }

  const response = await androidHttpRequest({
    url: url.toString(),
    method: 'GET',
    headers,
    body: null
  })

  if (response.status < 200 || response.status > 299) {
    throw new Error(`HTTP ${response.status}: ${response.body.slice(0, 800)}`)
  }

  return JSON.parse(response.body)
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
  httpRequest: androidHttpRequest,
  httpGetJson,
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
