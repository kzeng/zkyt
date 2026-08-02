<template>
  <div>
    <ft-loader
      v-if="isLoading"
      :fullscreen="true"
    />
    <ft-card
      v-else
      class="card"
    >
      <h2>
        <FontAwesomeIcon
          :icon="['fas', 'users']"
          class="headingIcon"
        />
        {{ $t("Most Popular") }}
      </h2>
      <ft-element-list
        :data="shownResults"
      />
    </ft-card>
    <ft-refresh-widget
      :disable-refresh="isLoading"
      :last-refresh-timestamp="lastPopularRefreshTimestamp"
      :title="$t('Most Popular')"
      @click="fetchPopularInfo"
    />
  </div>
</template>

<script setup>
import { FontAwesomeIcon } from '@fortawesome/vue-fontawesome'
import { computed, onBeforeUnmount, onMounted, ref, shallowRef } from 'vue'

import FtLoader from '../../components/FtLoader/FtLoader.vue'
import FtCard from '../../components/ft-card/ft-card.vue'
import FtElementList from '../../components/FtElementList/FtElementList.vue'
import FtRefreshWidget from '../../components/FtRefreshWidget/FtRefreshWidget.vue'
import store from '../../store/index'

import { getInvidiousPopularFeed } from '../../helpers/api/invidious'
import { getLocalSearchResults, getLocalTrending } from '../../helpers/api/local'
import { copyToClipboard, getRelativeTimeFromDate, showToast } from '../../helpers/utils'
import { useI18n } from 'vue-i18n'
import { KeyboardShortcuts } from '../../../constants'

const { t } = useI18n()

const isLoading = ref(false)

const lastPopularRefreshTimestamp = computed(() => {
  return getRelativeTimeFromDate(store.getters.getLastPopularRefreshTimestamp, true)
})

/** @type {import('vue').ComputedRef<Array | null>} */
const popularCache = computed(() => {
  return store.getters.getPopularCache
})

const shownResults = shallowRef(popularCache.value || [])

const backendPreference = computed(() => {
  return store.getters.getBackendPreference
})

const backendFallback = computed(() => {
  return store.getters.getBackendFallback
})

const region = computed(() => {
  return store.getters.getRegion.toUpperCase()
})

onMounted(() => {
  document.addEventListener('keydown', keyboardShortcutHandler)

  if (shownResults.value.length === 0) {
    fetchPopularInfo()
  }
})

onBeforeUnmount(() => {
  document.removeEventListener('keydown', keyboardShortcutHandler)
})

async function fetchPopularInfo() {
  isLoading.value = true

  try {
    const items = await getPopularFeed()

    store.commit('setLastPopularRefreshTimestamp', new Date())
    shownResults.value = items
    isLoading.value = false
    store.commit('setPopularCache', items)
  } catch (err) {
    isLoading.value = false
    const errorMessage = t('Invidious API Error (Click to copy)')
    showToast(`${errorMessage}: ${err}`, 10000, () => {
      copyToClipboard(err)
    })
  }
}

async function getPopularFeed() {
  if (process.env.SUPPORTS_LOCAL_API && (backendPreference.value === 'local' || process.env.IS_TAURI)) {
    try {
      const items = await getLocalTrending(region.value)
      if (items.length > 0 || !backendFallback.value) {
        return items
      }

      const { results } = await getLocalSearchResults('popular videos', {
        type: 'video'
      }, false)
      if (results.length > 0 || !backendFallback.value) {
        return results
      }
    } catch (error) {
      console.warn('Local popular feed failed, falling back to Invidious', error)
      if (!backendFallback.value) {
        throw error
      }
    }
  }

  return await getInvidiousPopularFeed()
}

/**
 * @param {KeyboardEvent} event the keyboard event
 */
function keyboardShortcutHandler(event) {
  if (document.activeElement.classList.contains('ft-input')) {
    return
  }
  // Avoid handling events due to user holding a key (not released)
  // https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/repeat
  if (event.repeat) { return }

  switch (event.key.toLowerCase()) {
    case 'f5':
    case KeyboardShortcuts.APP.SITUATIONAL.REFRESH:
      if (!isLoading.value) {
        fetchPopularInfo()
      }
      break
  }
}

</script>
<style scoped src="./Popular.css" />
