import { DBActions } from '../../constants'
import platform from '../../renderer/platform'

class Settings {
  static find() {
    return platform.dbSettings(DBActions.GENERAL.FIND)
  }

  static upsert(_id, value) {
    return platform.dbSettings(DBActions.GENERAL.UPSERT, { _id, value })
  }
}

class History {
  static find() {
    return platform.dbHistory(DBActions.GENERAL.FIND)
  }

  static upsert(record) {
    return platform.dbHistory(DBActions.GENERAL.UPSERT, record)
  }

  static overwrite(records) {
    return platform.dbHistory(DBActions.GENERAL.OVERWRITE, records)
  }

  static updateWatchProgress(videoId, watchProgress) {
    return platform.dbHistory(
      DBActions.HISTORY.UPDATE_WATCH_PROGRESS,
      { videoId, watchProgress }
    )
  }

  static updateLastViewedPlaylist(videoId, lastViewedPlaylistId, lastViewedPlaylistType, lastViewedPlaylistItemId) {
    return platform.dbHistory(
      DBActions.HISTORY.UPDATE_PLAYLIST,
      { videoId, lastViewedPlaylistId, lastViewedPlaylistType, lastViewedPlaylistItemId }
    )
  }

  static delete(videoId) {
    return platform.dbHistory(DBActions.GENERAL.DELETE, videoId)
  }

  static deleteAll() {
    return platform.dbHistory(DBActions.GENERAL.DELETE_ALL)
  }
}

class Profiles {
  static create(profile) {
    return platform.dbProfiles(DBActions.GENERAL.CREATE, profile)
  }

  static find() {
    return platform.dbProfiles(DBActions.GENERAL.FIND)
  }

  static upsert(profile) {
    return platform.dbProfiles(DBActions.GENERAL.UPSERT, profile)
  }

  static addChannelToProfiles(channel, profileIds) {
    return platform.dbProfiles(DBActions.PROFILES.ADD_CHANNEL, { channel, profileIds })
  }

  static removeChannelFromProfiles(channelId, profileIds) {
    return platform.dbProfiles(DBActions.PROFILES.REMOVE_CHANNEL, { channelId, profileIds })
  }

  static delete(id) {
    return platform.dbProfiles(DBActions.GENERAL.DELETE, id)
  }
}

class Playlists {
  static create(playlists) {
    return platform.dbPlaylists(DBActions.GENERAL.CREATE, playlists)
  }

  static find() {
    return platform.dbPlaylists(DBActions.GENERAL.FIND)
  }

  static upsert(playlist) {
    return platform.dbPlaylists(DBActions.GENERAL.UPSERT, playlist)
  }

  static upsertVideoByPlaylistId(_id, lastUpdatedAt, videoData) {
    return platform.dbPlaylists(
      DBActions.PLAYLISTS.UPSERT_VIDEO,
      { _id, lastUpdatedAt, videoData }
    )
  }

  static upsertVideosByPlaylistId(_id, lastUpdatedAt, videos) {
    return platform.dbPlaylists(
      DBActions.PLAYLISTS.UPSERT_VIDEOS,
      { _id, lastUpdatedAt, videos }
    )
  }

  static delete(_id) {
    return platform.dbPlaylists(DBActions.GENERAL.DELETE, _id)
  }

  static deleteVideoIdByPlaylistId(_id, lastUpdatedAt, videoId, playlistItemId) {
    return platform.dbPlaylists(
      DBActions.PLAYLISTS.DELETE_VIDEO_ID,
      { _id, lastUpdatedAt, videoId, playlistItemId }
    )
  }

  static deleteVideoIdsByPlaylistId(_id, lastUpdatedAt, playlistItemIds) {
    return platform.dbPlaylists(
      DBActions.PLAYLISTS.DELETE_VIDEO_IDS,
      { _id, lastUpdatedAt, playlistItemIds }
    )
  }

  static deleteAllVideosByPlaylistId(_id) {
    return platform.dbPlaylists(DBActions.PLAYLISTS.DELETE_ALL_VIDEOS, _id)
  }

  static deleteMultiple(ids) {
    return platform.dbPlaylists(DBActions.GENERAL.DELETE_MULTIPLE, ids)
  }

  static deleteAll() {
    return platform.dbPlaylists(DBActions.GENERAL.DELETE_ALL)
  }
}

class SearchHistory {
  static find() {
    return platform.dbSearchHistory(DBActions.GENERAL.FIND)
  }

  static upsert(searchHistoryEntry) {
    return platform.dbSearchHistory(DBActions.GENERAL.UPSERT, searchHistoryEntry)
  }

  static overwrite(records) {
    return platform.dbSearchHistory(DBActions.GENERAL.OVERWRITE, records)
  }

  static delete(_id) {
    return platform.dbSearchHistory(DBActions.GENERAL.DELETE, _id)
  }

  static deleteAll() {
    return platform.dbSearchHistory(DBActions.GENERAL.DELETE_ALL)
  }
}

class SubscriptionCache {
  static find() {
    return platform.dbSubscriptionCache(DBActions.GENERAL.FIND)
  }

  static updateVideosByChannelId(channelId, entries, timestamp) {
    return platform.dbSubscriptionCache(
      DBActions.SUBSCRIPTION_CACHE.UPDATE_VIDEOS_BY_CHANNEL,
      { channelId, entries, timestamp }
    )
  }

  static updateLiveStreamsByChannelId(channelId, entries, timestamp) {
    return platform.dbSubscriptionCache(
      DBActions.SUBSCRIPTION_CACHE.UPDATE_LIVE_STREAMS_BY_CHANNEL,
      { channelId, entries, timestamp }
    )
  }

  static updateShortsByChannelId(channelId, entries, timestamp) {
    return platform.dbSubscriptionCache(
      DBActions.SUBSCRIPTION_CACHE.UPDATE_SHORTS_BY_CHANNEL,
      { channelId, entries, timestamp }
    )
  }

  static updateShortsWithChannelPageShortsByChannelId(channelId, entries) {
    return platform.dbSubscriptionCache(
      DBActions.SUBSCRIPTION_CACHE.UPDATE_SHORTS_WITH_CHANNEL_PAGE_SHORTS_BY_CHANNEL,
      { channelId, entries }
    )
  }

  static updateCommunityPostsByChannelId(channelId, entries, timestamp) {
    return platform.dbSubscriptionCache(
      DBActions.SUBSCRIPTION_CACHE.UPDATE_COMMUNITY_POSTS_BY_CHANNEL,
      { channelId, entries, timestamp }
    )
  }

  static deleteMultipleChannels(channelIds) {
    return platform.dbSubscriptionCache(DBActions.GENERAL.DELETE_MULTIPLE, channelIds)
  }

  static deleteAll() {
    return platform.dbSubscriptionCache(DBActions.GENERAL.DELETE_ALL)
  }
}

export {
  Settings as settings,
  History as history,
  Profiles as profiles,
  Playlists as playlists,
  SearchHistory as searchHistory,
  SubscriptionCache as subscriptionCache,
}
