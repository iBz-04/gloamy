import { useColorMode } from '@vueuse/core'
import { MotionPlugin } from '@vueuse/motion'
import { createPinia } from 'pinia'
import { createApp } from 'vue'

import { useSettingsStore } from '@/stores/settings'
import { useUIStateStore } from '@/stores/uiState'
import { preloadAllTextFonts } from '@/lib/fonts'
import App from './App.vue'
import i18n from './i18n'
import router from './router'
import './assets/css/base.css'

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
app.use(router)
app.use(i18n)
app.use(MotionPlugin)

initAppSettings()

app.mount('#app')

async function initAppSettings() {
  const settingsStore = useSettingsStore()
  const uiStateStore = useUIStateStore()
  const mode = useColorMode()

  // Load persisted states
  await uiStateStore.loadState()

  const theme = await settingsStore.getSetting<string>('theme') as 'light' | 'dark' | 'auto'
  if (!theme) {
    settingsStore.setSetting('theme', mode.value)
  }
  else {
    mode.value = theme
  }

  const language = await settingsStore.getSetting<string>('language')
  if (language)
    i18n.global.locale.value = language

  // Preload text fonts in background (non-blocking)
  preloadAllTextFonts().catch(err => {
    console.warn('Failed to preload some fonts:', err)
  })
}
