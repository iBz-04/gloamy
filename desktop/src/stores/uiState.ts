import { LazyStore } from '@tauri-apps/plugin-store'
import { defineStore } from 'pinia'
import { ref, watch } from 'vue'

const store: LazyStore = new LazyStore('ui-state.json')

const UI_STATE_KEY = '__ui_state__'

export interface UIState {
  // Sidebar
  sidebarWidth: number
  sidebarTab: 'design' | 'advanced'
  leftSidebarCollapsed: boolean
  // Expanded sections
  expandedSections: Record<string, boolean>
  // Canvas
  lastZoomLevel: number
}

function createDefaultUIState(): UIState {
  return {
    sidebarWidth: 280,
    sidebarTab: 'design',
    leftSidebarCollapsed: false,
    expandedSections: {
      settings: true,
    },
    lastZoomLevel: 100,
  }
}

export const useUIStateStore = defineStore('uiState', () => {
  const state = ref<UIState>(createDefaultUIState())
  const isLoaded = ref(false)

  // Load state from persistent storage
  async function loadState() {
    try {
      const saved = await store.get<UIState>(UI_STATE_KEY)
      if (saved) {
        // Merge with defaults to handle new properties
        state.value = { ...createDefaultUIState(), ...saved }
      }
      isLoaded.value = true
    }
    catch (error) {
      console.error('Failed to load UI state:', error)
      isLoaded.value = true
    }
  }

  // Save state to persistent storage
  async function saveState() {
    try {
      await store.set(UI_STATE_KEY, state.value)
      await store.save()
    }
    catch (error) {
      console.error('Failed to save UI state:', error)
    }
  }

  // Watch for changes and auto-save (debounced)
  let saveTimeout: ReturnType<typeof setTimeout> | null = null
  watch(
    state,
    () => {
      if (!isLoaded.value)
        return
      if (saveTimeout)
        clearTimeout(saveTimeout)
      saveTimeout = setTimeout(saveState, 300)
    },
    { deep: true },
  )

  // Sidebar width
  function setSidebarWidth(width: number) {
    state.value.sidebarWidth = Math.max(280, Math.min(600, width))
  }

  // Sidebar tab
  function setSidebarTab(tab: 'design' | 'advanced') {
    state.value.sidebarTab = tab
  }

  // Left sidebar collapsed
  function setLeftSidebarCollapsed(collapsed: boolean) {
    state.value.leftSidebarCollapsed = collapsed
  }

  function toggleLeftSidebar() {
    state.value.leftSidebarCollapsed = !state.value.leftSidebarCollapsed
  }

  // Expanded sections
  function toggleSection(section: string) {
    state.value.expandedSections[section] = !state.value.expandedSections[section]
  }

  function setExpandedSection(section: string, expanded: boolean) {
    state.value.expandedSections[section] = expanded
  }

  // Zoom level
  function setLastZoomLevel(zoom: number) {
    state.value.lastZoomLevel = zoom
  }

  return {
    state,
    isLoaded,
    loadState,
    saveState,
    setSidebarWidth,
    setSidebarTab,
    setLeftSidebarCollapsed,
    toggleLeftSidebar,
    toggleSection,
    setExpandedSection,
    setLastZoomLevel,
  }
})
