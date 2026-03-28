<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAuthStore } from '@/stores/auth'

interface ConfigEntry {
  key: string
  value: string
  type: 'string' | 'number' | 'boolean' | 'array' | 'object'
  section: string
  rawLine: string
  lineIndex: number
}

interface ConfigSection {
  name: string
  icon: string
  description: string
  entries: ConfigEntry[]
}

const auth = useAuthStore()
const loading = ref(true)
const saving = ref(false)
const error = ref<string | null>(null)
const saveSuccess = ref(false)
const rawConfig = ref('')
const originalConfig = ref('')
const searchQuery = ref('')
const activeSection = ref('All')
const editingValueCell = ref<string | null>(null)
const editDraft = ref('')

const sectionMeta: Record<string, { icon: string; description: string }> = {
  '_root': { icon: 'hugeicons:settings-01', description: 'Core settings' },
  'observability': { icon: 'hugeicons:chart-bar-line', description: 'Logging and tracing' },
  'autonomy': { icon: 'hugeicons:robot-01', description: 'Agent autonomy levels' },
  'security': { icon: 'hugeicons:shield-01', description: 'Security policies' },
  'runtime': { icon: 'hugeicons:computer-terminal-01', description: 'Execution environment' },
  'reliability': { icon: 'hugeicons:refresh', description: 'Retries and fallbacks' },
  'scheduler': { icon: 'hugeicons:clock-01', description: 'Task scheduling' },
  'agent': { icon: 'hugeicons:brain', description: 'Agent orchestration' },
  'skills': { icon: 'hugeicons:sparkles', description: 'Skills loading' },
  'heartbeat': { icon: 'hugeicons:pulse-rectangle-01', description: 'Health pings' },
  'cron': { icon: 'hugeicons:calendar-01', description: 'Cron jobs' },
  'channels_config': { icon: 'hugeicons:chat-01', description: 'Channel settings' },
  'memory': { icon: 'hugeicons:database', description: 'Memory backends' },
  'storage': { icon: 'hugeicons:hard-drive', description: 'Persistent storage' },
  'tunnel': { icon: 'hugeicons:globe', description: 'Public exposure' },
  'gateway': { icon: 'hugeicons:plug-01', description: 'Gateway server' },
  'composio': { icon: 'hugeicons:puzzle', description: 'Composio integration' },
  'one': { icon: 'hugeicons:computer-terminal-02', description: 'One CLI integration' },
  'secrets': { icon: 'hugeicons:key-01', description: 'Secrets encryption' },
  'browser': { icon: 'hugeicons:browser', description: 'Browser automation' },
  'http_request': { icon: 'hugeicons:globe-02', description: 'HTTP requests' },
  'multimodal': { icon: 'hugeicons:image-01', description: 'Image handling' },
  'web_fetch': { icon: 'hugeicons:download-01', description: 'Web fetching' },
  'web_search': { icon: 'hugeicons:search-01', description: 'Web search' },
  'proxy': { icon: 'hugeicons:split', description: 'Proxy settings' },
  'identity': { icon: 'hugeicons:identification', description: 'Identity format' },
  'cost': { icon: 'hugeicons:wallet-01', description: 'Cost tracking' },
  'peripherals': { icon: 'hugeicons:cpu', description: 'Hardware boards' },
  'hardware': { icon: 'hugeicons:chip', description: 'Hardware config' },
  'transcription': { icon: 'hugeicons:mic-01', description: 'Voice transcription' },
  'agents': { icon: 'hugeicons:user-group', description: 'Sub-agent delegates' },
  'hooks': { icon: 'hugeicons:anchor', description: 'Lifecycle hooks' },
}


const hasChanges = computed(() => rawConfig.value !== originalConfig.value)

function cellId(entry: ConfigEntry): string {
  return `${entry.section}:${entry.lineIndex}`
}

function isApiLikeKey(key: string): boolean {
  return key.toLowerCase().includes('api')
}

function isMaskedPlaceholder(value: string): boolean {
  return value.includes('***') || value.includes('•••')
}

function displayValue(entry: ConfigEntry): string {
  if (isApiLikeKey(entry.key)) {
    return '••••••••'
  }
  return entry.value
}

function startValueEdit(entry: ConfigEntry) {
  editingValueCell.value = cellId(entry)
  editDraft.value = isMaskedPlaceholder(entry.value) ? '' : entry.value
}

function clearEditing() {
  editingValueCell.value = null
  editDraft.value = ''
}

function detectType(value: string): ConfigEntry['type'] {
  if (value === 'true' || value === 'false') return 'boolean'
  if (value.startsWith('[')) return 'array'
  if (value.startsWith('{')) return 'object'
  if (/^-?\d+(\.\d+)?$/.test(value)) return 'number'
  return 'string'
}

function parseValue(raw: string): string {
  const trimmed = raw.trim()
  if (trimmed.startsWith('"') && trimmed.endsWith('"')) {
    return trimmed.slice(1, -1)
  }
  if (trimmed.startsWith("'") && trimmed.endsWith("'")) {
    return trimmed.slice(1, -1)
  }
  return trimmed
}

const showAddModal = ref(false)
const newEntryKey = ref('')
const newEntryValue = ref('')
const newEntrySection = ref('General')
const newEntrySectionValue = ref('')

function openAddModal() {
  newEntryKey.value = ''
  newEntryValue.value = ''
  newEntrySectionValue.value = ''
  newEntrySection.value = activeSection.value === 'All' ? 'General' : activeSection.value
  showAddModal.value = true
}

function addNewEntry() {
  if (!newEntryKey.value.trim()) return

  const sectionToUse = newEntrySection.value === 'New Section' ? newEntrySectionValue.value : newEntrySection.value
  if (!sectionToUse) return

  const lines = rawConfig.value.split('\n')
  const targetHeader = `[${sectionToUse}]`
  
  let insertionIndex = -1
  if (sectionToUse === 'General') {
    insertionIndex = lines.findIndex(l => l.trim() !== '' && !l.trim().startsWith('#') && !l.trim().startsWith('['))
    if (insertionIndex === -1) insertionIndex = lines.length
  } else {
    const headerIndex = lines.findIndex(l => l.trim() === targetHeader)
    if (headerIndex !== -1) {
      insertionIndex = lines.findIndex((l, i) => i > headerIndex && l.trim().startsWith('['))
      if (insertionIndex === -1) insertionIndex = lines.length
    } else {
      lines.push('', targetHeader)
      insertionIndex = lines.length
    }
  }

  const formattedValue = detectType(newEntryValue.value) === 'string' && !newEntryValue.value.startsWith('"')
    ? `"${newEntryValue.value}"`
    : newEntryValue.value

  lines.splice(insertionIndex, 0, `${newEntryKey.value} = ${formattedValue}`)
  rawConfig.value = lines.join('\n')
  showAddModal.value = false
}

const sections = computed<ConfigSection[]>(() => {
  const lines = rawConfig.value.split('\n')
  const sectionsMap: Record<string, ConfigEntry[]> = { '_root': [] }
  let currentSection = '_root'

  lines.forEach((line, index) => {
    const trimmed = line.trim()
    
    // Section header
    const sectionMatch = trimmed.match(/^\[([^\]]+)\]$/)
    if (sectionMatch && sectionMatch[1]) {
      currentSection = sectionMatch[1]
      if (!sectionsMap[currentSection]) {
        sectionsMap[currentSection] = []
      }
      return
    }

    // Skip comments and empty lines
    if (trimmed.startsWith('#') || trimmed === '') return

    // Key-value pair
    const kvMatch = trimmed.match(/^([^=]+)=(.*)$/)
    if (kvMatch && kvMatch[1] && kvMatch[2] !== undefined) {
      const key = kvMatch[1].trim()
      const rawValue = kvMatch[2].trim()
      const value = parseValue(rawValue)
      
      if (!sectionsMap[currentSection]) {
        sectionsMap[currentSection] = []
      }
      
      const entries = sectionsMap[currentSection]
      if (entries) {
        entries.push({
          key,
          value,
          type: detectType(rawValue),
          section: currentSection,
          rawLine: line,
          lineIndex: index,
        })
      }
    }
  })

  const result: ConfigSection[] = []
  
  for (const [name, entries] of Object.entries(sectionsMap)) {
    if (entries.length === 0) continue
    const meta = sectionMeta[name] || { icon: 'hugeicons:folder-01', description: 'Configuration section' }
    result.push({
      name: name === '_root' ? 'General' : name,
      icon: meta.icon,
      description: meta.description,
      entries,
    })
  }

  return result
})

const sectionNames = computed(() => {
  return ['All', ...sections.value.map(s => s.name)]
})

const filteredSections = computed(() => {
  let result = sections.value

  if (activeSection.value !== 'All') {
    result = result.filter(s => s.name === activeSection.value)
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.map(section => ({
      ...section,
      entries: section.entries.filter(e =>
        e.key.toLowerCase().includes(q) ||
        e.value.toLowerCase().includes(q)
      ),
    })).filter(s => s.entries.length > 0)
  }

  return result
})

function updateEntry(entry: ConfigEntry, newValue: string) {
  const lines = rawConfig.value.split('\n')
  
  // Format value based on type
  let formattedValue = newValue
  if (entry.type === 'string' && !newValue.startsWith('"') && !newValue.startsWith('[') && !newValue.startsWith('{')) {
    formattedValue = `"${newValue}"`
  }
  
  // Replace the line
  const newLine = `${entry.key} = ${formattedValue}`
  lines[entry.lineIndex] = newLine
  rawConfig.value = lines.join('\n')

  clearEditing()
}

function toggleBoolean(entry: ConfigEntry) {
  const newValue = entry.value === 'true' ? 'false' : 'true'
  const lines = rawConfig.value.split('\n')
  lines[entry.lineIndex] = `${entry.key} = ${newValue}`
  rawConfig.value = lines.join('\n')
}

function getTypeIcon(type: ConfigEntry['type']): string {
  switch (type) {
    case 'string': return 'hugeicons:text'
    case 'number': return 'hugeicons:hashtag'
    case 'boolean': return 'hugeicons:toggle-off'
    case 'array': return 'hugeicons:list-view'
    case 'object': return 'hugeicons:code'
    default: return 'hugeicons:question'
  }
}

function getTypeColor(type: ConfigEntry['type']): string {
  switch (type) {
    case 'string': return 'text-sky-500'
    case 'number': return 'text-violet-500'
    case 'boolean': return 'text-emerald-500'
    case 'array': return 'text-amber-500'
    case 'object': return 'text-rose-500'
    default: return 'text-muted-foreground'
  }
}

async function fetchConfig() {
  loading.value = true
  error.value = null
  try {
    const response = await auth.fetchWithAuth<{ format: string; content: string }>('/api/config')
    rawConfig.value = response.content
    originalConfig.value = response.content
  } catch (err: any) {
    error.value = err.message || 'Failed to load configuration'
  } finally {
    loading.value = false
  }
}

async function saveConfig() {
  saving.value = true
  error.value = null
  saveSuccess.value = false
  try {
    await auth.fetchWithAuth('/api/config', {
      method: 'PUT',
      headers: { 'Content-Type': 'text/plain' },
      body: rawConfig.value,
    })
    originalConfig.value = rawConfig.value
    saveSuccess.value = true
    setTimeout(() => {
      saveSuccess.value = false
    }, 3000)
  } catch (err: any) {
    error.value = err.message || 'Failed to save configuration'
  } finally {
    saving.value = false
  }
}

function resetChanges() {
  rawConfig.value = originalConfig.value
}

function handleKeyDown(event: KeyboardEvent) {
  if ((event.metaKey || event.ctrlKey) && event.key === 's') {
    event.preventDefault()
    if (hasChanges.value && !saving.value) {
      saveConfig()
    }
  }
}

onMounted(() => {
  fetchConfig()
  window.addEventListener('keydown', handleKeyDown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown)
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <!-- Header -->
    <div class="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border/30">
      <div class="flex items-center justify-between mb-4">
        <div class="relative max-w-md">
          <Icon icon="hugeicons:search-01" class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search configuration..."
            class="w-full pl-10 pr-4 py-2 text-[13px] bg-card/50 border border-border/50 rounded-xl focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground"
          />
        </div>

        <div class="flex items-center gap-2">
          <button
            v-if="hasChanges"
            @click="resetChanges"
            class="px-3 py-1.5 text-[12px] font-medium rounded-lg text-muted-foreground hover:text-foreground hover:bg-card/50 transition-colors"
          >
            Discard
          </button>
          <button
            @click="saveConfig"
            :disabled="!hasChanges || saving"
            class="px-4 py-1.5 text-[12px] font-medium rounded-lg transition-all flex items-center gap-1.5"
            :class="hasChanges
              ? 'bg-foreground text-background hover:opacity-90'
              : 'bg-card/50 text-muted-foreground cursor-not-allowed'"
          >
            <Icon v-if="saving" icon="hugeicons:loading-03" class="size-3.5 animate-spin" />
            <Icon v-else-if="saveSuccess" icon="hugeicons:tick-02" class="size-3.5" />
            <Icon v-else icon="hugeicons:floppy-disk" class="size-3.5" />
            {{ saving ? 'Saving...' : saveSuccess ? 'Saved' : 'Save' }}
          </button>
          
          <button
            @click="openAddModal"
            class="px-4 py-1.5 text-[12px] font-medium rounded-lg bg-card/50 text-foreground border border-border/50 hover:bg-card transition-all flex items-center gap-1.5"
          >
            <Icon icon="hugeicons:add-01" class="size-3.5" />
            New Entry
          </button>
        </div>
      </div>

      <!-- Section tabs -->
      <div class="flex items-center gap-1 overflow-x-auto pb-1">
        <button
          v-for="name in sectionNames"
          :key="name"
          @click="activeSection = name"
          class="px-3 py-1.5 text-[12px] font-medium rounded-xl whitespace-nowrap transition-colors"
          :class="activeSection === name
            ? 'text-foreground'
            : 'text-muted-foreground hover:text-foreground hover:bg-card/50'"
        >
          {{ name }}
        </button>
      </div>
    </div>

    <!-- Content -->
    <div class="flex-1 overflow-hidden">
      <div v-if="loading" class="flex-1 flex items-center justify-center h-full">
        <Icon icon="hugeicons:loading-03" class="size-6 animate-spin text-muted-foreground" />
      </div>

      <div v-else-if="error && !rawConfig" class="flex-1 flex items-center justify-center h-full px-6">
        <div class="max-w-md w-full p-6 rounded-lg border border-border/50 bg-destructive/5 text-center">
          <Icon icon="hugeicons:alert-01" class="size-8 text-destructive mx-auto mb-3" />
          <h3 class="text-lg font-medium text-foreground mb-2">Failed to load configuration</h3>
          <p class="text-sm text-muted-foreground mb-4">{{ error }}</p>
          <button
            @click="fetchConfig"
            class="px-4 py-2 bg-primary text-primary-foreground rounded-lg text-sm font-medium hover:opacity-90 transition-opacity"
          >
            Try Again
          </button>
        </div>
      </div>

      <div v-else class="h-full overflow-auto">
        <!-- Error banner -->
        <div v-if="error" class="sticky top-0 z-10 px-6 py-2 bg-destructive/10 border-b border-destructive/20">
          <div class="flex items-center gap-2 text-[13px] text-destructive">
            <Icon icon="hugeicons:alert-01" class="size-4" />
            <span>{{ error }}</span>
            <button @click="error = null" class="ml-auto hover:opacity-70">
              <Icon icon="hugeicons:cancel-01" class="size-4" />
            </button>
          </div>
        </div>

        <!-- Table -->
        <div class="px-6 py-4">
          <div v-if="filteredSections.length === 0" class="flex items-center justify-center py-12">
            <p class="text-muted-foreground text-[13px]">No configuration entries found.</p>
          </div>

          <div v-else class="space-y-6">
            <section v-for="section in filteredSections" :key="section.name">
              <div class="flex items-center gap-2 mb-3">
                <Icon :icon="section.icon" class="size-4 text-muted-foreground" />
                <h2 class="text-[16px] font-medium text-foreground">{{ section.name }}</h2>
                <span class="text-[12px] text-muted-foreground">{{ section.description }}</span>
              </div>

              <!-- Table structure -->
              <div class="rounded-xl border border-border/40 overflow-hidden bg-card/20">
                <!-- Header row -->
                <div class="grid grid-cols-[1fr_80px_2fr] gap-4 px-4 py-2.5 bg-card/40 border-b border-border/30 text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
                  <div>Key</div>
                  <div>Type</div>
                  <div>Value</div>
                </div>

                <!-- Data rows -->
                <div
                  v-for="entry in section.entries"
                  :key="`${section.name}-${entry.key}`"
                  class="grid grid-cols-[1fr_80px_2fr] gap-4 px-4 py-3 border-b border-border/20 last:border-0 hover:bg-card/30 transition-colors group"
                >
                  <!-- Key -->
                  <div class="flex items-center gap-2 min-w-0">
                    <span class="font-mono text-[13px] text-foreground truncate" :title="entry.key">
                      {{ entry.key }}
                    </span>
                  </div>

                  <!-- Type badge -->
                  <div class="flex items-center">
                    <span
                      class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-medium bg-card/60"
                      :class="getTypeColor(entry.type)"
                    >
                      <Icon :icon="getTypeIcon(entry.type)" class="size-3" />
                      {{ entry.type }}
                    </span>
                  </div>

                  <!-- Value -->
                  <div class="flex items-center min-w-0">
                    <!-- Boolean toggle -->
                    <template v-if="entry.type === 'boolean'">
                      <button
                        @click="toggleBoolean(entry)"
                        class="flex items-center gap-2 px-3 py-1 rounded-lg transition-colors hover:bg-card/40"
                        :class="entry.value === 'true' ? 'text-emerald-600' : 'text-muted-foreground'"
                      >
                        <Icon
                          :icon="entry.value === 'true' ? 'hugeicons:toggle-on' : 'hugeicons:toggle-off'"
                          class="size-5"
                        />
                        <span class="text-[13px] font-medium">{{ entry.value }}</span>
                      </button>
                    </template>

                    <!-- Editable text input -->
                    <template v-else-if="editingValueCell === cellId(entry)">
                      <input
                        v-model="editDraft"
                        @blur="updateEntry(entry, editDraft)"
                        @keydown.enter.prevent="updateEntry(entry, editDraft)"
                        @keydown.escape.prevent="clearEditing"
                        autofocus
                        class="flex-1 px-3 py-1.5 font-mono text-[13px] bg-card/60 border border-primary/50 rounded-lg focus:outline-none focus:ring-1 focus:ring-primary/50 text-foreground"
                      />
                    </template>

                    <!-- Display value (click to edit) -->
                    <template v-else>
                      <button
                        @click="startValueEdit(entry)"
                        class="flex-1 text-left px-3 py-1.5 font-mono text-[13px] text-foreground bg-transparent hover:bg-card/40 rounded-lg transition-colors truncate group-hover:bg-card/40"
                      >
                        <span v-if="isApiLikeKey(entry.key) || (entry.type === 'string' && isMaskedPlaceholder(entry.value))" class="text-muted-foreground italic">
                          {{ displayValue(entry) }}
                        </span>
                        <span v-else-if="entry.type === 'array' || entry.type === 'object'" class="text-muted-foreground">
                          {{ displayValue(entry).length > 50 ? displayValue(entry).slice(0, 50) + '...' : displayValue(entry) }}
                        </span>
                        <span v-else>{{ displayValue(entry) || '(empty)' }}</span>
                      </button>
                      <button
                        @click="startValueEdit(entry)"
                        class="ml-2 p-1 rounded text-muted-foreground hover:text-foreground hover:bg-card/50 opacity-0 group-hover:opacity-100 transition-all flex-shrink-0"
                        aria-label="Edit value"
                        title="Edit value"
                      >
                        <Icon
                          icon="hugeicons:pencil-edit-01"
                          class="size-3.5"
                        />
                      </button>
                    </template>
                  </div>
                </div>
              </div>
            </section>
          </div>
        </div>

        <!-- Footer -->
        <div class="sticky bottom-0 px-6 py-2 border-t border-border/30 bg-background/95 backdrop-blur flex items-center justify-between text-[11px] text-muted-foreground">
          <div class="flex items-center gap-4">
            <span class="flex items-center gap-1">
              <Icon icon="hugeicons:keyboard" class="size-3.5" />
              <kbd class="px-1.5 py-0.5 bg-card/60 rounded text-[10px]">⌘S</kbd> to save
            </span>
            <span v-if="hasChanges" class="flex items-center gap-1 text-amber-600">
              <Icon icon="hugeicons:pencil-edit-01" class="size-3" />
              Unsaved changes
            </span>
          </div>
          <div class="flex items-center gap-3">
            <span>{{ sections.reduce((acc, s) => acc + s.entries.length, 0) }} entries</span>
            <span>{{ sections.length }} sections</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Add Entry Modal -->
    <Teleport to="body">
      <div
        v-if="showAddModal"
        class="fixed inset-0 z-50 flex items-end sm:items-center justify-center bg-black/50"
        @click.self="showAddModal = false"
      >
        <div class="w-full max-w-sm mx-4 mb-4 sm:mb-0 bg-card border border-border/60 rounded-2xl shadow-2xl overflow-hidden">
          <!-- Header -->
          <div class="flex items-center justify-between px-5 py-4 border-b border-border/40">
            <span class="text-[14px] font-semibold text-foreground">New Config Entry</span>
            <button
              @click="showAddModal = false"
              class="size-7 flex items-center justify-center rounded-lg hover:bg-muted/50 transition-colors"
            >
              <Icon icon="hugeicons:cancel-01" class="size-4 text-muted-foreground" />
            </button>
          </div>

          <!-- Body -->
          <div class="px-5 py-4 space-y-3">
            <!-- Section -->
            <div class="space-y-1">
              <label class="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Section</label>
              <input
                v-model="newEntrySection"
                list="section-suggestions"
                type="text"
                placeholder="e.g. agents.coder"
                class="w-full px-3 py-2 text-[13px] font-mono bg-background border border-border/60 rounded-lg focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground/40"
              />
              <datalist id="section-suggestions">
                <option v-for="name in sectionNames.filter(n => n !== 'All')" :key="name" :value="name" />
              </datalist>
            </div>

            <!-- Key -->
            <div class="space-y-1">
              <label class="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Key</label>
              <input
                v-model="newEntryKey"
                type="text"
                placeholder="e.g. system_prompt"
                autofocus
                class="w-full px-3 py-2 text-[13px] font-mono bg-background border border-border/60 rounded-lg focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground/40"
              />
            </div>

            <!-- Value -->
            <div class="space-y-1">
              <label class="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Value</label>
              <input
                v-model="newEntryValue"
                type="text"
                placeholder="true, 42, or &quot;some text&quot;"
                @keydown.enter="addNewEntry"
                class="w-full px-3 py-2 text-[13px] font-mono bg-background border border-border/60 rounded-lg focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground/40"
              />
            </div>
          </div>

          <!-- Footer -->
          <div class="flex items-center justify-end gap-2 px-5 py-4 border-t border-border/40">
            <button
              @click="showAddModal = false"
              class="px-4 py-1.5 text-[13px] text-muted-foreground hover:text-foreground hover:bg-muted/40 rounded-lg transition-colors"
            >
              Cancel
            </button>
            <button
              @click="addNewEntry"
              :disabled="!newEntryKey.trim()"
              class="px-4 py-1.5 text-[13px] font-semibold bg-foreground text-background rounded-lg hover:opacity-90 disabled:opacity-30 disabled:cursor-not-allowed transition-all"
            >
              Add
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
kbd {
  font-family: inherit;
}
</style>
