<script setup lang="ts">
import { ref, computed } from 'vue'
import { Icon } from '@iconify/vue'

interface MemoryEntry {
  id: string
  key: string
  value: string
  category: 'core' | 'daily' | 'conversation' | 'preference' | 'experience' | 'self_learning'
  createdAt: Date
  updatedAt: Date
  size: number
  pinned?: boolean
}

const memories = ref<MemoryEntry[]>([
  {
    id: '1',
    key: 'user_name',
    value: 'Ibrahim prefers to be called Ibz',
    category: 'core',
    createdAt: new Date('2024-01-15'),
    updatedAt: new Date('2024-07-14T19:09:00'),
    size: 156,
    pinned: true,
  },
  {
    id: '2',
    key: 'coding_style',
    value: 'User prefers minimal, focused code changes with clear explanations',
    category: 'preference',
    createdAt: new Date('2024-02-20'),
    updatedAt: new Date('2024-07-14T14:01:00'),
    size: 312,
    pinned: true,
  },
  {
    id: '3',
    key: 'project_context',
    value: 'Working on Gloamy - a Rust-first autonomous agent runtime',
    category: 'core',
    createdAt: new Date('2024-03-10'),
    updatedAt: new Date('2024-05-09T12:01:00'),
    size: 1024,
    pinned: true,
  },
  {
    id: '4',
    key: 'timezone',
    value: 'User timezone is UTC+03:00',
    category: 'preference',
    createdAt: new Date('2024-01-20'),
    updatedAt: new Date('2024-05-09T12:01:00'),
    size: 89,
  },
  {
    id: '5',
    key: 'daily_standup',
    value: 'Completed code review for PR #142, fixed memory leak in provider',
    category: 'daily',
    createdAt: new Date('2024-07-14'),
    updatedAt: new Date('2024-07-14T10:05:00'),
    size: 2048,
  },
  {
    id: '6',
    key: 'conversation_summary',
    value: 'Discussed implementing new Memory page UI for desktop app',
    category: 'conversation',
    createdAt: new Date('2024-07-14'),
    updatedAt: new Date('2024-07-14T23:01:00'),
    size: 4096,
  },
  {
    id: '7',
    key: 'api_preferences',
    value: 'Prefers OpenAI for complex tasks, Claude for code review',
    category: 'preference',
    createdAt: new Date('2024-04-05'),
    updatedAt: new Date('2024-07-14T19:09:00'),
    size: 256,
  },
  {
    id: '8',
    key: 'workspace_path',
    value: '/Users/ibz/Desktop/zeroclaw',
    category: 'core',
    createdAt: new Date('2024-01-10'),
    updatedAt: new Date('2024-07-18T14:01:00'),
    size: 128,
  },
  {
    id: '9',
    key: 'debug_session',
    value: 'Debugging Telegram channel connection issue - resolved by removing active_workspace.toml',
    category: 'conversation',
    createdAt: new Date('2024-07-12'),
    updatedAt: new Date('2024-07-18T14:01:00'),
    size: 512,
  },
  {
    id: '10',
    key: 'ui_preferences',
    value: 'Desktop app should use modern UI with TailwindCSS, pill-shaped tags, split auth screen',
    category: 'preference',
    createdAt: new Date('2024-06-20'),
    updatedAt: new Date('2024-07-18T14:01:00'),
    size: 384,
  },
  {
    id: '11',
    key: 'agent_experience_google_docs',
    value: 'When a doc was just created, return the existing doc link instead of creating a duplicate doc.',
    category: 'experience',
    createdAt: new Date('2026-03-20'),
    updatedAt: new Date('2026-03-20T18:15:00'),
    size: 148,
  },
  {
    id: '12',
    key: 'self_learning_telegram_link_format',
    value: 'In Telegram replies, prefer explicit clickable URL formatting when sending generated document links.',
    category: 'self_learning',
    createdAt: new Date('2026-03-22'),
    updatedAt: new Date('2026-03-22T09:45:00'),
    size: 162,
  },
])

const categories = computed(() => {
  const cats = new Set(memories.value.map(m => m.category))
  return ['All', ...Array.from(cats).sort()]
})

const activeCategory = ref('All')
const searchQuery = ref('')
const sortBy = ref<'updatedAt' | 'key' | 'size'>('updatedAt')
const sortOrder = ref<'asc' | 'desc'>('desc')
const selectedMemories = ref<Set<string>>(new Set())
const editingMemoryId = ref<string | null>(null)
const editingKey = ref('')
const editingValue = ref('')
const editingCategory = ref<MemoryEntry['category']>('core')
const memoryCategoryOptions: MemoryEntry['category'][] = ['core', 'daily', 'conversation', 'preference', 'experience', 'self_learning']
const actionMenuMemoryId = ref<string | null>(null)
const deleteConfirmMemoryId = ref<string | null>(null)

const pinnedMemories = computed(() => {
  return memories.value.filter(m => m.pinned).slice(0, 5)
})

const filteredMemories = computed(() => {
  let result = memories.value

  if (activeCategory.value !== 'All') {
    result = result.filter(m => m.category === activeCategory.value)
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(m =>
      m.key.toLowerCase().includes(q) ||
      m.value.toLowerCase().includes(q)
    )
  }

  // Sort
  result = [...result].sort((a, b) => {
    let cmp = 0
    if (sortBy.value === 'updatedAt') {
      cmp = a.updatedAt.getTime() - b.updatedAt.getTime()
    } else if (sortBy.value === 'key') {
      cmp = a.key.localeCompare(b.key)
    } else if (sortBy.value === 'size') {
      cmp = a.size - b.size
    }
    return sortOrder.value === 'desc' ? -cmp : cmp
  })

  return result
})

const deleteTargetMemory = computed(() => {
  if (!deleteConfirmMemoryId.value) {
    return null
  }
  return memories.value.find(memory => memory.id === deleteConfirmMemoryId.value) ?? null
})

function formatDate(date: Date): string {
  const day = date.getDate()
  const month = date.getMonth() + 1
  const year = date.getFullYear()
  const hours = date.getHours()
  const minutes = date.getMinutes().toString().padStart(2, '0')
  const ampm = hours >= 12 ? 'pm' : 'am'
  const hour12 = hours % 12 || 12
  return `${day}/${month}/${year}  ${hour12}:${minutes} ${ampm}`
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`
}

function formatCategoryLabel(category: string): string {
  return category
    .split('_')
    .map(part => part.charAt(0).toUpperCase() + part.slice(1))
    .join(' ')
}

function categoryIcon(category: string): string {
  switch (category) {
    case 'core': return 'ph:user-circle-fill'
    case 'daily': return 'ph:calendar-fill'
    case 'conversation': return 'ph:chat-circle-text-fill'
    case 'preference': return 'ph:gear-six-fill'
    case 'experience': return 'ph:graduation-cap-fill'
    case 'self_learning': return 'ph:books-fill'
    default: return 'ph:file-fill'
  }
}

function categoryColor(category: string): string {
  switch (category) {
    case 'core': return 'text-blue-500'
    case 'daily': return 'text-amber-500'
    case 'conversation': return 'text-emerald-500'
    case 'preference': return 'text-purple-500'
    case 'experience': return 'text-cyan-500'
    case 'self_learning': return 'text-fuchsia-500'
    default: return 'text-muted-foreground'
  }
}

function toggleSort(column: 'updatedAt' | 'key' | 'size') {
  if (sortBy.value === column) {
    sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  } else {
    sortBy.value = column
    sortOrder.value = 'desc'
  }
}

function toggleSelect(id: string) {
  if (selectedMemories.value.has(id)) {
    selectedMemories.value.delete(id)
  } else {
    selectedMemories.value.add(id)
  }
}

function toggleSelectAll() {
  if (selectedMemories.value.size === filteredMemories.value.length) {
    selectedMemories.value.clear()
  } else {
    selectedMemories.value = new Set(filteredMemories.value.map(m => m.id))
  }
}

function copyMemory(memory: MemoryEntry) {
  navigator.clipboard.writeText(memory.value)
}

function deleteMemory(id: string) {
  if (editingMemoryId.value === id) {
    cancelEdit()
  }
  if (actionMenuMemoryId.value === id) {
    actionMenuMemoryId.value = null
  }
  if (deleteConfirmMemoryId.value === id) {
    deleteConfirmMemoryId.value = null
  }
  memories.value = memories.value.filter(m => m.id !== id)
  selectedMemories.value.delete(id)
}

function togglePin(memory: MemoryEntry) {
  memory.pinned = !memory.pinned
}

function startEdit(memory: MemoryEntry) {
  actionMenuMemoryId.value = null
  editingMemoryId.value = memory.id
  editingKey.value = memory.key
  editingValue.value = memory.value
  editingCategory.value = memory.category
}

function cancelEdit() {
  editingMemoryId.value = null
  editingKey.value = ''
  editingValue.value = ''
  editingCategory.value = 'core'
}

function saveEdit(id: string) {
  const key = editingKey.value.trim()
  const value = editingValue.value.trim()

  if (!key || !value) {
    return
  }

  const memory = memories.value.find(item => item.id === id)
  if (!memory) {
    cancelEdit()
    return
  }

  memory.key = key
  memory.value = value
  memory.category = editingCategory.value
  memory.updatedAt = new Date()
  memory.size = new TextEncoder().encode(value).length

  cancelEdit()
}

function toggleActionMenu(id: string) {
  if (actionMenuMemoryId.value === id) {
    actionMenuMemoryId.value = null
    return
  }
  actionMenuMemoryId.value = id
}

function openDeleteConfirmModal(id: string) {
  deleteConfirmMemoryId.value = id
  actionMenuMemoryId.value = null
}

function closeDeleteConfirmModal() {
  deleteConfirmMemoryId.value = null
}

function confirmDeleteFromModal() {
  if (!deleteConfirmMemoryId.value) {
    return
  }
  deleteMemory(deleteConfirmMemoryId.value)
  deleteConfirmMemoryId.value = null
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <!-- Header with search and filters -->
    <div class="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border/30">
      <!-- Search -->
      <div class="relative mb-4 max-w-md">
        <Icon icon="ph:magnifying-glass" class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search memories..."
          class="w-full pl-10 pr-4 py-2 text-[13px] bg-card/50 border border-border/50 rounded-xl focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground"
        />
      </div>

      <!-- Category tabs -->
      <div class="flex items-center gap-1 overflow-x-auto pb-1">
        <button
          v-for="cat in categories"
          :key="cat"
          @click="activeCategory = cat"
          class="px-3 py-1.5 text-[12px] font-medium rounded-full whitespace-nowrap transition-colors"
          :class="activeCategory === cat
            ? 'bg-foreground text-background'
            : 'text-muted-foreground hover:text-foreground hover:bg-card/50'"
        >
          {{ cat === 'All' ? 'All' : formatCategoryLabel(cat) }}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-y-auto px-6 py-4">
      <!-- Pinned / Recently Used Section -->
      <section v-if="pinnedMemories.length > 0 && activeCategory === 'All' && !searchQuery" class="mb-6">
        <h2 class="text-[12px] font-medium text-muted-foreground uppercase tracking-wider mb-3">Pinned</h2>
        <div class="flex gap-3 overflow-x-auto pb-2">
          <div
            v-for="memory in pinnedMemories"
            :key="memory.id"
            class="group relative flex-shrink-0 w-[140px] h-[120px] rounded-xl border border-border/40 bg-card/30 hover:bg-card/50 hover:border-border/60 transition-all cursor-pointer p-3 flex flex-col"
          >
            <!-- Menu button -->
            <button class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-foreground">
              <Icon icon="ph:dots-three-bold" class="size-4" />
            </button>

            <!-- Icon -->
            <div class="size-10 rounded-lg bg-muted/30 flex items-center justify-center mb-2">
              <Icon :icon="categoryIcon(memory.category)" :class="['size-5', categoryColor(memory.category)]" />
            </div>

            <!-- Title -->
            <p class="text-[13px] font-medium text-foreground truncate">{{ memory.key }}</p>
            <p class="text-[11px] text-muted-foreground truncate mt-0.5">{{ formatCategoryLabel(memory.category) }}</p>
          </div>
        </div>
      </section>

      <!-- All Memories Table -->
      <section>
        <h2 class="text-[12px] font-medium text-muted-foreground uppercase tracking-wider mb-3">All memories</h2>

        <div v-if="filteredMemories.length === 0" class="flex items-center justify-center py-12">
          <p class="text-muted-foreground text-[13px]">No memories found.</p>
        </div>

        <div v-else class="border border-border/40 rounded-xl overflow-hidden">
          <!-- Table Header -->
          <div class="grid grid-cols-[auto_1fr_180px_100px_120px] gap-4 px-4 py-3 bg-card/30 border-b border-border/30 text-[12px] font-medium text-muted-foreground">
            <div class="flex items-center">
              <button
                @click="toggleSelectAll"
                class="size-4 rounded border border-border/60 flex items-center justify-center hover:border-foreground/50 transition-colors"
                :class="selectedMemories.size === filteredMemories.length && filteredMemories.length > 0 ? 'bg-primary border-primary' : ''"
              >
                <Icon v-if="selectedMemories.size === filteredMemories.length && filteredMemories.length > 0" icon="ph:check-bold" class="size-3 text-primary-foreground" />
              </button>
            </div>
            <button @click="toggleSort('key')" class="flex items-center gap-1 hover:text-foreground transition-colors text-left">
              Name
              <Icon v-if="sortBy === 'key'" :icon="sortOrder === 'asc' ? 'ph:caret-up' : 'ph:caret-down'" class="size-3" />
            </button>
            <button @click="toggleSort('updatedAt')" class="flex items-center gap-1 hover:text-foreground transition-colors">
              Last modified
              <Icon v-if="sortBy === 'updatedAt'" :icon="sortOrder === 'asc' ? 'ph:caret-up' : 'ph:caret-down'" class="size-3" />
            </button>
            <button @click="toggleSort('size')" class="flex items-center gap-1 hover:text-foreground transition-colors">
              Size
              <Icon v-if="sortBy === 'size'" :icon="sortOrder === 'asc' ? 'ph:caret-up' : 'ph:caret-down'" class="size-3" />
            </button>
            <div class="text-right">Manage</div>
          </div>

          <!-- Table Rows -->
          <div
            v-for="memory in filteredMemories"
            :key="memory.id"
            class="grid grid-cols-[auto_1fr_180px_100px_120px] gap-4 px-4 py-3 border-b border-border/20 last:border-b-0 hover:bg-card/20 transition-colors items-center text-[13px]"
          >
            <!-- Checkbox -->
            <div class="flex items-center">
              <button
                @click="toggleSelect(memory.id)"
                class="size-4 rounded border border-border/60 flex items-center justify-center hover:border-foreground/50 transition-colors"
                :class="selectedMemories.has(memory.id) ? 'bg-primary border-primary' : ''"
              >
                <Icon v-if="selectedMemories.has(memory.id)" icon="ph:check-bold" class="size-3 text-primary-foreground" />
              </button>
            </div>

            <!-- Name with icon -->
            <div class="flex items-center gap-3 min-w-0">
              <Icon :icon="categoryIcon(memory.category)" :class="['size-4 flex-shrink-0', categoryColor(memory.category)]" />
              <div v-if="editingMemoryId === memory.id" class="min-w-0 flex-1 space-y-1">
                <input
                  v-model="editingKey"
                  type="text"
                  class="w-full px-2 py-1 text-[12px] bg-card/60 border border-border/50 rounded-md focus:outline-none focus:ring-1 focus:ring-primary/50"
                  placeholder="Memory key"
                />
                <input
                  v-model="editingValue"
                  type="text"
                  class="w-full px-2 py-1 text-[12px] bg-card/60 border border-border/50 rounded-md focus:outline-none focus:ring-1 focus:ring-primary/50"
                  placeholder="Memory value"
                />
                <select
                  v-model="editingCategory"
                  class="w-full px-2 py-1 text-[12px] bg-card/60 border border-border/50 rounded-md focus:outline-none focus:ring-1 focus:ring-primary/50"
                >
                  <option v-for="option in memoryCategoryOptions" :key="option" :value="option">
                    {{ formatCategoryLabel(option) }}
                  </option>
                </select>
              </div>
              <div v-else class="min-w-0">
                <p class="text-foreground font-medium truncate">{{ memory.key }}</p>
                <p class="text-[11px] text-muted-foreground truncate">{{ memory.value }}</p>
              </div>
            </div>

            <!-- Last modified -->
            <div class="text-muted-foreground text-[12px]">
              {{ formatDate(memory.updatedAt) }}
            </div>

            <!-- Size -->
            <div class="text-muted-foreground text-[12px]">
              {{ formatSize(memory.size) }}
            </div>

            <!-- Actions -->
            <div class="relative flex items-center justify-end gap-1">
              <template v-if="editingMemoryId === memory.id">
                <button
                  @click="saveEdit(memory.id)"
                  class="px-2.5 py-1 text-[11px] font-medium border border-border/50 rounded-lg hover:bg-card/50 transition-colors flex items-center gap-1"
                >
                  <Icon icon="ph:floppy-disk" class="size-3" />
                  Save
                </button>
                <button
                  @click="cancelEdit"
                  class="px-2.5 py-1 text-[11px] font-medium border border-border/50 rounded-lg hover:bg-card/50 transition-colors flex items-center gap-1"
                >
                  <Icon icon="ph:x" class="size-3" />
                  Cancel
                </button>
              </template>
              <template v-else>
                <button
                  @click="copyMemory(memory)"
                  class="px-2.5 py-1 text-[11px] font-medium border border-border/50 rounded-lg hover:bg-card/50 transition-colors flex items-center gap-1"
                >
                  <Icon icon="ph:copy" class="size-3" />
                  Copy
                </button>
                <button
                  @click="togglePin(memory)"
                  class="size-7 flex items-center justify-center rounded-lg hover:bg-card/50 transition-colors"
                  :class="memory.pinned ? 'text-amber-500' : 'text-muted-foreground'"
                  :title="memory.pinned ? 'Unpin' : 'Pin'"
                >
                  <Icon :icon="memory.pinned ? 'ph:push-pin-fill' : 'ph:push-pin'" class="size-4" />
                </button>
                <button
                  @click="toggleActionMenu(memory.id)"
                  class="size-7 flex items-center justify-center rounded-lg hover:bg-card/50 text-muted-foreground hover:text-foreground transition-colors"
                  title="More actions"
                >
                  <Icon icon="ph:dots-three-bold" class="size-4" />
                </button>
                <div
                  v-if="actionMenuMemoryId === memory.id"
                  class="absolute right-0 top-9 z-20 w-28 rounded-lg border border-border/60 bg-card p-1 shadow-lg"
                >
                  <button
                    @click="startEdit(memory)"
                    class="w-full px-2 py-1.5 text-left text-[12px] rounded-md hover:bg-card/80 transition-colors flex items-center gap-1.5"
                  >
                    <Icon icon="ph:pencil-simple" class="size-3.5" />
                    Edit
                  </button>
                  <button
                    @click="openDeleteConfirmModal(memory.id)"
                    class="w-full px-2 py-1.5 text-left text-[12px] rounded-md hover:bg-destructive/10 text-destructive transition-colors flex items-center gap-1.5"
                  >
                    <Icon icon="ph:trash" class="size-3.5" />
                    Delete
                  </button>
                </div>
              </template>
            </div>
          </div>
        </div>
      </section>
    </div>
    <Teleport to="body">
      <div
        v-if="deleteConfirmMemoryId"
        class="fixed inset-0 z-50 flex items-center justify-center bg-black/45 px-4"
        @click.self="closeDeleteConfirmModal"
      >
        <div class="w-full max-w-sm rounded-xl border border-border/60 bg-background p-4 shadow-2xl">
          <h3 class="text-[15px] font-semibold text-foreground">Delete memory?</h3>
          <p class="mt-2 text-[13px] text-muted-foreground">
            This will permanently remove
            <span class="font-medium text-foreground">{{ deleteTargetMemory?.key || 'this memory' }}</span>.
          </p>
          <div class="mt-4 flex items-center justify-end gap-2">
            <button
              @click="closeDeleteConfirmModal"
              class="px-3 py-1.5 text-[12px] font-medium border border-border/60 rounded-lg hover:bg-card/60 transition-colors"
            >
              Cancel
            </button>
            <button
              @click="confirmDeleteFromModal"
              class="px-3 py-1.5 text-[12px] font-medium rounded-lg bg-destructive text-destructive-foreground hover:opacity-90 transition-opacity"
            >
              Delete
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>
