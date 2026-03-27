<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { computed, onMounted, ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

interface MemoryEntry {
  id: string
  key: string
  content: string
  category: string
  timestamp: string
  session_id: string | null
  score: number | null
}

const auth = useAuthStore()
const entries = ref<MemoryEntry[]>([])
const loading = ref(true)
const saving = ref(false)
const error = ref<string | null>(null)
const searchQuery = ref('')
const activeCategory = ref('All')

const showAddModal = ref(false)
const formKey = ref('')
const formContent = ref('')
const formCategory = ref('')
const formError = ref<string | null>(null)

const deleteTarget = ref<MemoryEntry | null>(null)

const categories = computed(() => {
  const names = Array.from(new Set(entries.value.map(entry => entry.category))).sort((a, b) => a.localeCompare(b))
  return ['All', ...names]
})

const filteredEntries = computed(() => {
  let result = entries.value

  if (activeCategory.value !== 'All') {
    result = result.filter(entry => entry.category === activeCategory.value)
  }

  const query = searchQuery.value.trim().toLowerCase()
  if (query) {
    result = result.filter(entry =>
      entry.key.toLowerCase().includes(query)
      || entry.content.toLowerCase().includes(query)
      || entry.category.toLowerCase().includes(query),
    )
  }

  return [...result].sort((a, b) => {
    const aTime = new Date(a.timestamp).getTime()
    const bTime = new Date(b.timestamp).getTime()
    return bTime - aTime
  })
})

function formatTimestamp(value: string): string {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return '—'
  }
  return date.toLocaleString()
}

function categoryIcon(category: string): string {
  const normalized = category.toLowerCase()
  if (normalized === 'core') {
    return 'ph:user-circle-fill'
  }
  if (normalized === 'daily') {
    return 'ph:calendar-fill'
  }
  if (normalized === 'conversation') {
    return 'ph:chat-circle-text-fill'
  }
  return 'ph:books-fill'
}

function categoryClass(category: string): string {
  const normalized = category.toLowerCase()
  if (normalized === 'core') {
    return 'text-blue-500'
  }
  if (normalized === 'daily') {
    return 'text-amber-500'
  }
  if (normalized === 'conversation') {
    return 'text-emerald-500'
  }
  return 'text-fuchsia-500'
}

function normalizeListResponse(response: MemoryEntry[] | { entries?: MemoryEntry[] }): MemoryEntry[] {
  if (Array.isArray(response)) {
    return response
  }
  if (response && Array.isArray(response.entries)) {
    return response.entries
  }
  return []
}

async function fetchEntries(showLoading = true) {
  if (showLoading) {
    loading.value = true
  }

  error.value = null

  try {
    const response = await auth.fetchWithAuth<MemoryEntry[] | { entries?: MemoryEntry[] }>('/api/memory')
    entries.value = normalizeListResponse(response)
  }
  catch (err: unknown) {
    error.value = err instanceof Error ? err.message : 'Failed to load memories'
    entries.value = []
  }
  finally {
    if (showLoading) {
      loading.value = false
    }
  }
}

function openAddModal() {
  formKey.value = ''
  formContent.value = ''
  formCategory.value = ''
  formError.value = null
  showAddModal.value = true
}

async function addMemory() {
  const key = formKey.value.trim()
  const content = formContent.value.trim()
  const category = formCategory.value.trim()

  if (!key || !content) {
    formError.value = 'Key and content are required.'
    return
  }

  saving.value = true
  formError.value = null

  try {
    await auth.fetchWithAuth('/api/memory', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        key,
        content,
        category: category || undefined,
      }),
    })

    showAddModal.value = false
    await fetchEntries(false)
  }
  catch (err: unknown) {
    formError.value = err instanceof Error ? err.message : 'Failed to store memory'
  }
  finally {
    saving.value = false
  }
}

function requestDelete(entry: MemoryEntry) {
  deleteTarget.value = entry
}

function closeDeleteModal() {
  deleteTarget.value = null
}

async function confirmDelete() {
  if (!deleteTarget.value) {
    return
  }

  const key = deleteTarget.value.key
  error.value = null

  try {
    await auth.fetchWithAuth(`/api/memory/${encodeURIComponent(key)}`, { method: 'DELETE' })
    entries.value = entries.value.filter(entry => entry.key !== key)
    deleteTarget.value = null
  }
  catch (err: unknown) {
    error.value = err instanceof Error ? err.message : 'Failed to delete memory'
  }
}

onMounted(() => {
  fetchEntries()
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <div class="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border/30 space-y-4">
      <div class="flex items-center gap-3">
        <div class="relative flex-1 max-w-xl">
          <Icon icon="ph:magnifying-glass" class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search memories..."
            class="w-full pl-10 pr-4 py-2 text-[13px] bg-card/50 border border-border/50 rounded-xl focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground"
          >
        </div>

        <button
          class="px-3 py-2 text-[12px] font-medium rounded-xl border border-border/60 hover:bg-card/60 transition-colors"
          @click="fetchEntries()"
        >
          Refresh
        </button>

        <button
          class="px-3 py-2 text-[12px] font-medium rounded-xl bg-foreground text-background hover:opacity-90 transition-opacity"
          @click="openAddModal"
        >
          Add Memory
        </button>
      </div>

      <div class="flex items-center gap-1 overflow-x-auto pb-1">
        <button
          v-for="cat in categories"
          :key="cat"
          class="px-3 py-1.5 text-[12px] font-medium rounded-full whitespace-nowrap transition-colors"
          :class="activeCategory === cat
            ? 'bg-foreground text-background'
            : 'text-muted-foreground hover:text-foreground hover:bg-card/50'"
          @click="activeCategory = cat"
        >
          {{ cat }}
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-6 py-4">
      <div v-if="loading" class="space-y-2">
        <div
          v-for="i in 8"
          :key="i"
          class="h-12 rounded-xl bg-card/30 border border-border/30 animate-pulse"
        />
      </div>

      <template v-else>
        <div v-if="error" class="mb-4 px-3 py-2 text-[12px] text-amber-500 flex items-center gap-2">
          <Icon icon="ph:warning" class="size-4" />
          <span>{{ error }}</span>
        </div>

        <div v-if="filteredEntries.length === 0" class="flex flex-col items-center justify-center py-20">
          <Icon icon="ph:archive" class="size-12 text-muted-foreground/30 mb-3" />
          <p class="text-muted-foreground text-[13px]">No memories found</p>
        </div>

        <div v-else class="border border-border/40 rounded-xl overflow-hidden">
          <div class="grid grid-cols-[220px_130px_180px_1fr_90px] gap-4 px-4 py-3 bg-card/30 border-b border-border/30 text-[12px] font-medium text-muted-foreground">
            <div>Key</div>
            <div>Category</div>
            <div>Updated</div>
            <div>Content</div>
            <div class="text-right">Actions</div>
          </div>

          <div
            v-for="entry in filteredEntries"
            :key="entry.id"
            class="grid grid-cols-[220px_130px_180px_1fr_90px] gap-4 px-4 py-3 border-b border-border/20 last:border-b-0 items-center text-[13px] hover:bg-card/20 transition-colors"
          >
            <div class="truncate text-foreground font-medium" :title="entry.key">
              {{ entry.key }}
            </div>

            <div class="inline-flex items-center gap-1.5 min-w-0">
              <Icon :icon="categoryIcon(entry.category)" class="size-3.5" :class="categoryClass(entry.category)" />
              <span class="truncate text-muted-foreground" :title="entry.category">{{ entry.category }}</span>
            </div>

            <div class="text-muted-foreground text-[12px]">
              {{ formatTimestamp(entry.timestamp) }}
            </div>

            <div class="truncate text-foreground font-mono text-[12px]" :title="entry.content">
              {{ entry.content }}
            </div>

            <div class="flex justify-end">
              <button
                class="px-2.5 py-1 text-[11px] font-medium border border-border/60 rounded-lg hover:bg-card/60 transition-colors"
                @click="requestDelete(entry)"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      </template>
    </div>

    <Teleport to="body">
      <div
        v-if="showAddModal"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/45 px-4"
      >
        <div class="w-full max-w-lg rounded-xl border border-border/60 bg-background p-4 shadow-2xl">
          <h3 class="text-[15px] font-semibold text-foreground">Add Memory</h3>

          <div v-if="formError" class="mt-3 px-3 py-2 text-[12px] text-amber-500 border border-amber-500/30 rounded-lg">
            {{ formError }}
          </div>

          <div class="mt-3 space-y-3">
            <input
              v-model="formKey"
              type="text"
              placeholder="Memory key"
              class="w-full px-3 py-2 text-[13px] bg-card/50 border border-border/50 rounded-lg focus:outline-none focus:ring-1 focus:ring-primary/50"
            >
            <textarea
              v-model="formContent"
              rows="4"
              placeholder="Memory content"
              class="w-full px-3 py-2 text-[13px] bg-card/50 border border-border/50 rounded-lg focus:outline-none focus:ring-1 focus:ring-primary/50 resize-y"
            />
            <input
              v-model="formCategory"
              type="text"
              placeholder="Category (optional, e.g. preference)"
              class="w-full px-3 py-2 text-[13px] bg-card/50 border border-border/50 rounded-lg focus:outline-none focus:ring-1 focus:ring-primary/50"
            >
          </div>

          <div class="mt-4 flex items-center justify-end gap-2">
            <button
              class="px-3 py-1.5 text-[12px] font-medium border border-border/60 rounded-lg hover:bg-card/60 transition-colors"
              @click="showAddModal = false"
            >
              Cancel
            </button>
            <button
              class="px-3 py-1.5 text-[12px] font-medium rounded-lg bg-foreground text-background hover:opacity-90 transition-opacity disabled:opacity-60"
              :disabled="saving"
              @click="addMemory"
            >
              {{ saving ? 'Saving...' : 'Save' }}
            </button>
          </div>
        </div>
      </div>
    </Teleport>

    <Teleport to="body">
      <div
        v-if="deleteTarget"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/45 px-4"
        @click.self="closeDeleteModal"
      >
        <div class="w-full max-w-sm rounded-xl border border-border/60 bg-background p-4 shadow-2xl">
          <h3 class="text-[15px] font-semibold text-foreground">Delete memory?</h3>
          <p class="mt-2 text-[13px] text-muted-foreground">
            This will permanently remove <span class="font-medium text-foreground">{{ deleteTarget?.key }}</span>.
          </p>
          <div class="mt-4 flex items-center justify-end gap-2">
            <button
              class="px-3 py-1.5 text-[12px] font-medium border border-border/60 rounded-lg hover:bg-card/60 transition-colors"
              @click="closeDeleteModal"
            >
              Cancel
            </button>
            <button
              class="px-3 py-1.5 text-[12px] font-medium rounded-lg bg-destructive text-destructive-foreground hover:opacity-90 transition-opacity"
              @click="confirmDelete"
            >
              Delete
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
