<script setup lang="ts">
import { computed, ref } from 'vue'
import { Icon } from '@iconify/vue'

interface IntegrationDefinition {
  name: string
  logo: string
  displayName: string
  description: string
  category: string
  status: 'Connected' | 'Available' | 'Needs Auth' | 'Disabled'
  lastSynced?: string
  favorite?: boolean
}

const integrations = ref<IntegrationDefinition[]>([
  {
    name: 'composio',
    logo: 'ph:puzzle-piece-fill',
    displayName: 'Composio',
    description: 'Managed OAuth integration layer for 1000+ apps via the built-in composio tool.',
    category: 'Integration Platforms',
    status: 'Available',
    favorite: true,
  },
  {
    name: 'one',
    logo: 'ph:terminal-window-fill',
    displayName: 'One CLI',
    description: 'Third-party platform bridge through the built-in one tool (200+ platforms).',
    category: 'Integration Platforms',
    status: 'Available',
    favorite: true,
  },
  {
    name: 'slack',
    logo: 'logos:slack-icon',
    displayName: 'Slack',
    description: 'Native Slack channel adapter for workspace messaging and routing.',
    category: 'Channels',
    status: 'Available',
    favorite: true,
  },
  {
    name: 'telegram',
    logo: 'logos:telegram',
    displayName: 'Telegram',
    description: 'Native Telegram channel with command, message, and bot workflow support.',
    category: 'Channels',
    status: 'Available',
  },
  {
    name: 'discord',
    logo: 'logos:discord-icon',
    displayName: 'Discord',
    description: 'Native Discord channel integration for servers and direct events.',
    category: 'Channels',
    status: 'Available',
  },
  {
    name: 'whatsapp',
    logo: 'logos:whatsapp-icon',
    displayName: 'WhatsApp',
    description: 'Native WhatsApp channel integration including webhook and web modes.',
    category: 'Channels',
    status: 'Available',
  },
  {
    name: 'signal',
    logo: 'ph:shield-check-fill',
    displayName: 'Signal',
    description: 'Native Signal channel for secure messaging workflows.',
    category: 'Channels',
    status: 'Available',
  },
  {
    name: 'matrix',
    logo: 'ph:grid-four-fill',
    displayName: 'Matrix',
    description: 'Native Matrix channel support for homeserver-based messaging.',
    category: 'Channels',
    status: 'Available',
  },
  {
    name: 'irc',
    logo: 'ph:chat-circle-text-fill',
    displayName: 'IRC',
    description: 'Native IRC channel integration for classic chat infrastructure.',
    category: 'Channels',
    status: 'Available',
  },
  {
    name: 'lark',
    logo: 'ph:paper-plane-tilt-fill',
    displayName: 'Lark / Feishu',
    description: 'Native Lark channel adapter for message and bot workflows.',
    category: 'Channels',
    status: 'Available',
  },
  {
    name: 'dingtalk',
    logo: 'ph:briefcase-fill',
    displayName: 'DingTalk',
    description: 'Native DingTalk channel integration for enterprise messaging.',
    category: 'Channels',
    status: 'Available',
  },
  {
    name: 'mattermost',
    logo: 'logos:mattermost-icon',
    displayName: 'Mattermost',
    description: 'Native Mattermost channel adapter for self-hosted team chat.',
    category: 'Channels',
    status: 'Available',
  },
  {
    name: 'pushover',
    logo: 'ph:megaphone-simple-fill',
    displayName: 'Pushover',
    description: 'Built-in push notification tool for mobile and desktop alerts.',
    category: 'Notifications',
    status: 'Available',
  },
])

const categories = computed(() => {
  const cats = new Set(integrations.value.map(i => i.category))
  return ['All', ...Array.from(cats).sort()]
})

const activeCategory = ref('All')
const searchQuery = ref('')

const filteredIntegrations = computed(() => {
  let result = integrations.value

  if (activeCategory.value !== 'All') {
    result = result.filter(i => i.category === activeCategory.value)
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(i =>
      i.displayName.toLowerCase().includes(q)
      || i.description.toLowerCase().includes(q)
      || i.name.toLowerCase().includes(q),
    )
  }

  return result
})

const groupedIntegrations = computed(() => {
  const groups: Record<string, IntegrationDefinition[]> = {}

  if (activeCategory.value === 'All') {
    const pinned = filteredIntegrations.value.filter(i => i.favorite)
    if (pinned.length > 0) {
      groups['Pinned Integrations'] = pinned
    }
  }

  for (const integration of filteredIntegrations.value) {
    if (!groups[integration.category]) {
      groups[integration.category] = []
    }

    const currentGroup = groups[integration.category] as IntegrationDefinition[]
    const pinnedGroup = groups['Pinned Integrations']

    if (!integration.favorite || activeCategory.value !== 'All') {
      currentGroup.push(integration)
    } else if (!pinnedGroup?.includes(integration)) {
      currentGroup.push(integration)
    }
  }

  for (const key of Object.keys(groups)) {
    const group = groups[key]
    if (group && group.length === 0) {
      delete groups[key]
    }
  }

  return groups
})

function toggleFavorite(integration: IntegrationDefinition) {
  integration.favorite = !integration.favorite
}

function statusIcon(status: IntegrationDefinition['status']): string {
  switch (status) {
    case 'Connected': return 'ph:check-circle-fill'
    case 'Available': return 'ph:plus-circle-fill'
    case 'Needs Auth': return 'ph:key-fill'
    case 'Disabled': return 'ph:prohibit-fill'
    default: return 'ph:circle-fill'
  }
}

function statusClass(status: IntegrationDefinition['status']): string {
  switch (status) {
    case 'Connected': return 'text-emerald-500'
    case 'Available': return 'text-sky-500'
    case 'Needs Auth': return 'text-amber-500'
    case 'Disabled': return 'text-muted-foreground'
    default: return 'text-muted-foreground'
  }
}
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <div class="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border/30">
      <div class="relative mb-4 max-w-md">
        <Icon icon="ph:magnifying-glass" class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Search integrations..."
          class="w-full pl-10 pr-4 py-2 text-[13px] bg-card/50 border border-border/50 rounded-xl focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground"
        >
      </div>

      <div class="flex items-center gap-1 overflow-x-auto pb-1">
        <button
          v-for="cat in categories"
          :key="cat"
          class="px-3 py-1.5 text-[12px] font-medium rounded-xl whitespace-nowrap transition-colors"
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
      <div v-if="Object.keys(groupedIntegrations).length === 0" class="flex items-center justify-center h-full">
        <p class="text-muted-foreground text-[12px]">No integrations found matching your search.</p>
      </div>

      <div v-else class="space-y-6">
        <section v-for="(integrationList, groupName) in groupedIntegrations" :key="groupName">
          <h2 class="font-sans text-[20px] font-medium text-foreground tracking-tight mb-3">{{ groupName }}</h2>
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
            <div
              v-for="integration in integrationList"
              :key="integration.name"
              class="group relative p-4 rounded-2xl border border-border/40 bg-card/20 hover:bg-card/40 hover:border-border/60 transition-all cursor-pointer"
            >
              <button
                class="absolute top-3 right-3 text-muted-foreground hover:text-amber-500 transition-colors"
                :class="{ 'text-amber-500': integration.favorite }"
                @click.stop="toggleFavorite(integration)"
              >
                <Icon :icon="integration.favorite ? 'ph:star-fill' : 'ph:star'" class="size-4" />
              </button>

              <div class="flex items-center gap-2 pr-6 mb-1">
                <Icon :icon="integration.logo" class="size-[18px] shrink-0" />
                <h3 class="font-sans text-[15px] font-medium text-foreground">
                  {{ integration.displayName }}
                </h3>
              </div>

              <p class="text-[13px] text-muted-foreground line-clamp-2 mb-3">
                {{ integration.description }}
              </p>

              <div class="flex items-center justify-between gap-2">
                <span
                  class="inline-flex items-center gap-1.5 text-[11px] font-medium"
                  :class="statusClass(integration.status)"
                >
                  <Icon :icon="statusIcon(integration.status)" class="size-3.5" />
                  {{ integration.status }}
                </span>
                <span v-if="integration.lastSynced" class="text-[11px] text-muted-foreground">
                  Synced {{ integration.lastSynced }}
                </span>
              </div>
            </div>
          </div>
        </section>
      </div>
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
