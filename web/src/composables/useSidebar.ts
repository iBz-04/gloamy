import { storeToRefs } from 'pinia'
import { computed, ref } from 'vue'
import { useUIStateStore } from '@/stores/uiState'

export function useSidebar() {
  const uiStore = useUIStateStore()
  const { state } = storeToRefs(uiStore)

  const isResizingSidebar = ref(false)

  // Use computed to always reflect the store state
  const sidebarWidth = computed(() => state.value.sidebarWidth)
  const sidebarTab = computed(() => state.value.sidebarTab)
  const expandedSections = computed(() => state.value.expandedSections)

  function startResize(_e: MouseEvent) {
    isResizingSidebar.value = true
    document.addEventListener('mousemove', handleResize)
    document.addEventListener('mouseup', stopResize)
    document.body.style.cursor = 'grabbing'
    document.body.style.userSelect = 'none'
  }

  function handleResize(e: MouseEvent) {
    if (!isResizingSidebar.value)
      return
    const newWidth = window.innerWidth - e.clientX
    const clampedWidth = Math.max(280, Math.min(600, newWidth))
    uiStore.setSidebarWidth(clampedWidth)
  }

  function stopResize() {
    isResizingSidebar.value = false
    document.removeEventListener('mousemove', handleResize)
    document.removeEventListener('mouseup', stopResize)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }

  function toggleSection(section: string) {
    uiStore.toggleSection(section)
  }

  function setSidebarTab(tab: 'design' | 'advanced') {
    uiStore.setSidebarTab(tab)
  }

  return {
    sidebarWidth,
    isResizingSidebar,
    sidebarTab,
    expandedSections,
    startResize,
    toggleSection,
    setSidebarTab,
  }
}
