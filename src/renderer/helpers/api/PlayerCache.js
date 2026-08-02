import platform from '../../platform'

export class PlayerCache {
  async get(key) {
    return await platform.playerCacheGet(key)
  }

  async set(key, value) {
    await platform.playerCacheSet(key, value)
  }

  async remove(_key) {
    // no-op; YouTube.js only uses remove for the OAuth credentials, but we don't use that in FreeTube
  }
}
