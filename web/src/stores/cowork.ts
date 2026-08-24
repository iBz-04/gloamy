import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useAuthStore } from './auth'

interface HistoryEntry {
  id: string
  title: string
  updated_at: string
  created_at: string
  message_count: number
}

interface SessionMessage {
  role: string
  content: string
}

export const useCoworkStore = defineStore('cowork', () => {
  const currentSessionId = ref<string>('default_desktop_session')
  const sessions = ref<{ id: string; label: string; icon: string }[]>([])
  const isLoading = ref(false)

  const auth = useAuthStore()

  async function fetchHistory() {
    if (isLoading.value) return
    isLoading.value = true
    try {
      const response = await auth.fetchWithAuth<{ history: HistoryEntry[] }>('/api/history')
      sessions.value = response.history.map(entry => ({
        id: entry.id,
        label: entry.title,
        icon: 'hugeicons:message-01',
      }))
    } catch (error) {
      console.error('Failed to fetch chat history:', error)
    } finally {
      isLoading.value = false
    }
  }

  async function loadSessionHistory(threadId: string): Promise<SessionMessage[]> {
    try {
      const response = await auth.fetchWithAuth<{ thread_id: string; messages: SessionMessage[] }>(`/api/session/${threadId}`)
      return response.messages
    } catch (error) {
      console.error('Failed to load session history:', error)
      return []
    }
  }

  function setSessionId(id: string) {
    currentSessionId.value = id
  }

  function addSession(id: string, label: string) {
    sessions.value.unshift({ id, label, icon: 'hugeicons:message-01' })
  }

  return {
    currentSessionId,
    sessions,
    isLoading,
    fetchHistory,
    loadSessionHistory,
    setSessionId,
    addSession,
  }
})

