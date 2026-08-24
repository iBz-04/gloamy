<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { computed, onMounted, ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

interface Integration {
  name: string
  description: string
  category: string
  status: 'Available' | 'Active' | 'ComingSoon'
}

interface SkillCatalogEntry {
  name: string
  description: string
  category?: string
  status?: 'Available' | 'Active' | 'ComingSoon'
}

const auth = useAuthStore()
const integrations = ref<Integration[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const searchQuery = ref('')
const activeCategory = ref('All')

const categories = computed(() => {
  const values = Array.from(new Set(integrations.value.map(item => item.category))).sort((a, b) => a.localeCompare(b))
  return ['All', ...values]
})

const filteredIntegrations = computed(() => {
  let result = integrations.value

  if (activeCategory.value !== 'All') {
    result = result.filter(item => item.category === activeCategory.value)
  }

  const query = searchQuery.value.trim().toLowerCase()
  if (query) {
    result = result.filter(item =>
      item.name.toLowerCase().includes(query)
      || item.description.toLowerCase().includes(query)
      || item.category.toLowerCase().includes(query),
    )
  }

  return result
})

const groupedIntegrations = computed(() => {
  const groups: Record<string, Integration[]> = {}

  for (const integration of filteredIntegrations.value) {
    if (!groups[integration.category]) {
      groups[integration.category] = []
    }
    const category = groups[integration.category]
    if (category) {
      category.push(integration)
    }
  }

  return Object.fromEntries(
    Object.entries(groups).sort(([a], [b]) => a.localeCompare(b)),
  ) as Record<string, Integration[]>
})

const categoryLabelMap: Record<string, string> = {
  chat: 'Chat Providers',
  chatproviders: 'Chat Providers',
  aimodel: 'AI Models',
  aimodels: 'AI Models',
  productivity: 'Productivity',
  musicaudio: 'Music & Audio',
  smarthome: 'Smart Home',
  toolsautomation: 'Tools & Automation',
  mediacreative: 'Media & Creative',
  social: 'Social',
  platform: 'Platforms',
  platforms: 'Platforms',
}

function normalizeCategoryKey(category: string): string {
  return category.replace(/[^a-z0-9]/gi, '').toLowerCase()
}

function formatCategoryLabel(category: string): string {
  const mappedLabel = categoryLabelMap[normalizeCategoryKey(category)]
  if (mappedLabel) {
    return mappedLabel
  }

  const spaced = category
    .replace(/[_-]+/g, ' ')
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2')
    .trim()
    .replace(/\s+/g, ' ')

  return spaced || category
}

function normalizeEntries(entries: Integration[]): Integration[] {
  return entries.map(item => ({
    ...item,
    category: formatCategoryLabel(item.category),
  }))
}

function normalizeIntegrationResponse(response: Integration[] | { integrations?: Integration[] }): Integration[] {
  if (Array.isArray(response)) {
    return normalizeEntries(response)
  }
  if (response && Array.isArray(response.integrations)) {
    return normalizeEntries(response.integrations)
  }
  return []
}

function normalizeSkillsResponse(response: SkillCatalogEntry[] | { skills?: SkillCatalogEntry[] }): Integration[] {
  const entries = Array.isArray(response)
    ? response
    : (response && Array.isArray(response.skills) ? response.skills : [])

  return normalizeEntries(entries.map(skill => ({
    name: skill.name,
    description: skill.description,
    category: skill.category ?? 'Skills',
    status: skill.status ?? 'Active',
  })))
}

function statusIcon(status: Integration['status']): string {
  if (status === 'Active') {
    return 'hugeicons:tick-02'
  }
  if (status === 'Available') {
    return 'hugeicons:add-01'
  }
  return 'hugeicons:clock-01'
}

function statusClass(status: Integration['status']): string {
  if (status === 'Active') {
    return 'text-emerald-500'
  }
  if (status === 'Available') {
    return 'text-sky-500'
  }
  return 'text-muted-foreground'
}

function statusLabel(status: Integration['status']): string {
  if (status === 'ComingSoon') {
    return 'Coming Soon'
  }
  return status
}

interface IntegrationLogo {
  icon: string
  color?: string
}

const integrationLogoMap: Record<string, IntegrationLogo> = {
  Telegram: { icon: 'simple-icons:telegram', color: '#26A5E4' },
  Discord: { icon: 'simple-icons:discord', color: '#5865F2' },
  Slack: { icon: 'simple-icons:slack', color: '#4A154B' },
  WhatsApp: { icon: 'simple-icons:whatsapp', color: '#25D366' },
  Signal: { icon: 'simple-icons:signal', color: '#3A76F0' },
  iMessage: { icon: 'simple-icons:imessage', color: '#34DA50' },
  'Microsoft Teams': { icon: 'simple-icons:microsoftteams', color: '#6264A7' },
  Matrix: { icon: 'simple-icons:matrix' },
  'Nextcloud Talk': { icon: 'simple-icons:nextcloud', color: '#0082C9' },
  Zalo: { icon: 'simple-icons:zalo', color: '#0068FF' },
  'QQ Official': { icon: 'simple-icons:qq', color: '#EB1923' },
  OpenRouter: { icon: 'simple-icons:openrouter', color: '#00A3FF' },
  Anthropic: { icon: 'simple-icons:anthropic' },
  OpenAI: { icon: 'simple-icons:openai' },
  Google: { icon: 'simple-icons:googlegemini', color: '#8E75B2' },
  xAI: { icon: 'simple-icons:x' },
  Mistral: { icon: 'simple-icons:mistralai', color: '#FF7000' },
  Ollama: { icon: 'simple-icons:ollama' },
  Perplexity: { icon: 'simple-icons:perplexity', color: '#1FB8CD' },
  'Hugging Face': { icon: 'simple-icons:huggingface', color: '#FFD21E' },
  'Vercel AI': { icon: 'simple-icons:vercel' },
  'Cloudflare AI': { icon: 'simple-icons:cloudflare', color: '#F38020' },
  MiniMax: { icon: 'simple-icons:minimax', color: '#F04E23' },
  'Amazon Bedrock': { icon: 'simple-icons:amazon', color: '#FF9900' },
  GitHub: { icon: 'simple-icons:github' },
  Notion: { icon: 'simple-icons:notion' },
  'Apple Notes': { icon: 'simple-icons:apple' },
  'Apple Reminders': { icon: 'simple-icons:apple' },
  Obsidian: { icon: 'simple-icons:obsidian', color: '#7C3AED' },
  Trello: { icon: 'simple-icons:trello', color: '#0052CC' },
  Linear: { icon: 'simple-icons:linear', color: '#5E6AD2' },
  Spotify: { icon: 'simple-icons:spotify', color: '#1DB954' },
  Sonos: { icon: 'simple-icons:sonos' },
  Shazam: { icon: 'simple-icons:shazam', color: '#0088FF' },
  'Home Assistant': { icon: 'simple-icons:homeassistant', color: '#18BCF2' },
  'Philips Hue': { icon: 'simple-icons:philipshue', color: '#0065D3' },
  Gmail: { icon: 'simple-icons:gmail', color: '#EA4335' },
  '1Password': { icon: 'simple-icons:1password', color: '#3B66BC' },
  Canvas: { icon: 'simple-icons:canvas', color: '#E72429' },
  'Twitter/X': { icon: 'simple-icons:x' },
  macOS: { icon: 'simple-icons:apple' },
  Linux: { icon: 'simple-icons:linux', color: '#FCC624' },
  Windows: { icon: 'simple-icons:windows', color: '#0078D4' },
  iOS: { icon: 'simple-icons:apple' },
  Android: { icon: 'simple-icons:android', color: '#3DDC84' },
}

const skillIconMatchers: Array<{ pattern: RegExp, logo: IntegrationLogo }> = [
  { pattern: /docx|word|document/, logo: { icon: 'simple-icons:microsoftword', color: '#2B579A' } },
  { pattern: /xlsx|excel|sheet|spreadsheet/, logo: { icon: 'simple-icons:microsoftexcel', color: '#217346' } },
  { pattern: /pptx|powerpoint|slides?|presentation/, logo: { icon: 'simple-icons:microsoftpowerpoint', color: '#B7472A' } },
  { pattern: /pdf/, logo: { icon: 'simple-icons:adobeacrobatreader', color: '#EC1C24' } },
  { pattern: /github/, logo: { icon: 'simple-icons:github' } },
  { pattern: /slack/, logo: { icon: 'simple-icons:slack', color: '#4A154B' } },
  { pattern: /notion/, logo: { icon: 'simple-icons:notion' } },
  { pattern: /vercel/, logo: { icon: 'simple-icons:vercel' } },
]

function resolveIcon(name: string, category: string): IntegrationLogo | undefined {
  if (category === 'Skills') {
    for (const matcher of skillIconMatchers) {
      if (matcher.pattern.test(name.toLowerCase())) {
        return matcher.logo
      }
    }
  }

  return integrationLogoMap[name]
}

function integrationIcon(name: string, category: string): string {
  const logo = resolveIcon(name, category)
  if (logo) {
    return logo.icon
  }
  if (category === 'Skills') {
    return 'hugeicons:airplay-line'
  }
  return 'hugeicons:puzzle'
}

function integrationIconClass(name: string, category: string): string {
  const logo = resolveIcon(name, category)
  if (!logo) {
    return 'text-primary'
  }
  return logo.color ? '' : 'text-foreground'
}

function integrationIconStyle(name: string, category: string): { color: string } | undefined {
  const color = resolveIcon(name, category)?.color
  return color ? { color } : undefined
}

async function fetchIntegrations(showLoading = true) {
  if (showLoading) {
    loading.value = true
  }

  error.value = null

  const [integrationsResult, skillsResult] = await Promise.allSettled([
    auth.fetchWithAuth<Integration[] | { integrations?: Integration[] }>('/api/integrations'),
    auth.fetchWithAuth<SkillCatalogEntry[] | { skills?: SkillCatalogEntry[] }>('/api/skills'),
  ])

  const nextEntries: Integration[] = []
  const nextErrors: string[] = []

  if (integrationsResult.status === 'fulfilled') {
    nextEntries.push(...normalizeIntegrationResponse(integrationsResult.value))
  } else {
    nextErrors.push('Failed to load integrations')
  }

  if (skillsResult.status === 'fulfilled') {
    nextEntries.push(...normalizeSkillsResponse(skillsResult.value))
  } else {
    // Backward compatibility: older daemons do not expose /api/skills yet.
    // Keep integrations usable and avoid surfacing a hard error for this optional section.
    console.warn('Skills catalog unavailable:', skillsResult.reason)
  }

  integrations.value = nextEntries
  error.value = nextErrors.length > 0 ? nextErrors.join(' · ') : null

  if (showLoading) {
    loading.value = false
  }
}

onMounted(() => {
  fetchIntegrations()
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <div class="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border/30 space-y-4">
      <div class="flex items-center gap-3">
        <div class="relative flex-1 max-w-xl">
          <Icon icon="hugeicons:search-01" class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search integrations and skills..."
            class="w-full pl-10 pr-4 py-2 text-[13px] bg-card/50 border border-border/50 rounded-xl focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground"
          >
        </div>

        <button
          class="px-3 py-2 text-[12px] font-medium rounded-xl border border-border/60 hover:bg-card/60 transition-colors"
          @click="fetchIntegrations()"
        >
          Refresh
        </button>
      </div>

      <div class="flex items-center gap-1 overflow-x-auto pb-1">
        <button
          v-for="cat in categories"
          :key="cat"
          class="px-3 py-1.5 text-[12px] font-medium rounded-xl whitespace-nowrap transition-colors"
          :class="activeCategory === cat
            ? 'text-foreground'
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
          class="h-20 rounded-xl bg-card/30 border border-border/30 animate-pulse"
        />
      </div>

      <template v-else>
        <div v-if="error" class="mb-4 px-3 py-2 text-[12px] text-amber-500 flex items-center gap-2">
          <Icon icon="hugeicons:alert-01" class="size-4" />
          <span>{{ error }}</span>
        </div>

        <div v-if="Object.keys(groupedIntegrations).length === 0" class="flex items-center justify-center h-full">
          <p class="text-muted-foreground text-[12px]">No integrations or skills found.</p>
        </div>

        <div v-else class="space-y-6">
          <section v-for="(integrationList, groupName) in groupedIntegrations" :key="groupName">
            <h2 class="font-sans text-[20px] font-medium text-foreground tracking-tight mb-3">{{ groupName }}</h2>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
              <div
                v-for="integration in integrationList"
                :key="integration.name"
                class="p-4 rounded-2xl border border-border/40 bg-card/20 hover:bg-card/40 hover:border-border/60 transition-all"
              >
                <div class="flex items-center gap-2 pr-2 mb-1">
                  <Icon
                    :icon="integrationIcon(integration.name, integration.category)"
                    class="size-[16px] shrink-0"
                    :class="integrationIconClass(integration.name, integration.category)"
                    :style="integrationIconStyle(integration.name, integration.category)"
                  />
                  <h3 class="font-sans text-[15px] font-medium text-foreground truncate">{{ integration.name }}</h3>
                </div>

                <p class="text-[13px] text-muted-foreground line-clamp-2 mb-3">
                  {{ integration.description }}
                </p>

                <div class="flex items-center justify-between gap-2">
                  <span
                    class="inline-flex items-center gap-1.5 text-[11px] font-medium"
                    :class="statusClass(integration.status)"
                  >
                    <Icon :icon="statusIcon(integration.status)" class="size-3.5 shrink-0" />
                    {{ statusLabel(integration.status) }}
                  </span>
                  <span class="text-[11px] text-muted-foreground truncate" :title="integration.category">
                    {{ integration.category }}
                  </span>
                </div>
              </div>
            </div>
          </section>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
