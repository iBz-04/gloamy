<script setup lang="ts">
import { ref, computed } from 'vue'
import { Icon } from '@iconify/vue'

interface ToolDefinition {
  name: string
  displayName: string
  description: string
  category: string
  outputType: 'Output' | 'Draft' | 'Action' | 'Query'
  steps?: number
  favorite?: boolean
}

const tools = ref<ToolDefinition[]>([
  // File & Search
  {
    name: 'file_read',
    displayName: 'Read File Contents',
    description: 'Read file contents with line numbers. Supports partial reading via offset and limit. Extracts text from PDF.',
    category: 'File & Search',
    outputType: 'Output',
    steps: 1,
  },
  {
    name: 'file_write',
    displayName: 'Write to File',
    description: 'Write contents to a file in the workspace. Creates new files or overwrites existing ones.',
    category: 'File & Search',
    outputType: 'Action',
    steps: 1,
  },
  {
    name: 'file_edit',
    displayName: 'Edit File',
    description: 'Apply targeted edits to a file using search and replace patterns. Safer than full rewrites.',
    category: 'File & Search',
    outputType: 'Action',
    steps: 1,
  },
  {
    name: 'glob_search',
    displayName: 'Search Files by Pattern',
    description: 'Search for files matching a glob pattern within the workspace. Returns sorted list of matching paths.',
    category: 'File & Search',
    outputType: 'Query',
    steps: 1,
  },
  {
    name: 'content_search',
    displayName: 'Search File Contents',
    description: 'Search file contents by regex pattern within the workspace. Supports ripgrep with grep fallback.',
    category: 'File & Search',
    outputType: 'Query',
    steps: 1,
  },
  {
    name: 'pdf_read',
    displayName: 'Extract PDF Text',
    description: 'Extract text content from PDF documents. Handles multi-page documents with page markers.',
    category: 'File & Search',
    outputType: 'Output',
    steps: 1,
  },

  // Shell & System
  {
    name: 'shell',
    displayName: 'Execute Shell Command',
    description: 'Execute a shell command in the workspace directory with sandboxing and timeout protection.',
    category: 'Shell & System',
    outputType: 'Action',
    steps: 1,
  },
  {
    name: 'screenshot',
    displayName: 'Capture Screenshot',
    description: 'Capture a screenshot of the current screen. Returns the file path and base64-encoded PNG data.',
    category: 'Shell & System',
    outputType: 'Output',
    steps: 1,
  },
  {
    name: 'image_info',
    displayName: 'Analyze Image',
    description: 'Extract metadata and information from image files including dimensions, format, and EXIF data.',
    category: 'Shell & System',
    outputType: 'Output',
    steps: 1,
  },

  // Memory
  {
    name: 'memory_store',
    displayName: 'Store Memory',
    description: 'Store a fact, preference, or note in long-term memory. Categories: core, daily, conversation.',
    category: 'Memory',
    outputType: 'Action',
    steps: 1,
    favorite: true,
  },
  {
    name: 'memory_recall',
    displayName: 'Recall Memory',
    description: 'Search long-term memory for relevant facts, preferences, or context. Returns scored results.',
    category: 'Memory',
    outputType: 'Query',
    steps: 1,
    favorite: true,
  },
  {
    name: 'memory_forget',
    displayName: 'Forget Memory',
    description: 'Remove a memory by key. Use to delete outdated facts or sensitive data.',
    category: 'Memory',
    outputType: 'Action',
    steps: 1,
  },

  // Browser & Web
  {
    name: 'browser',
    displayName: 'Browser Automation',
    description: 'Web/browser automation with pluggable backends. Supports DOM actions plus optional OS-level actions.',
    category: 'Browser & Web',
    outputType: 'Action',
    steps: 2,
  },
  {
    name: 'browser_open',
    displayName: 'Open URL in Browser',
    description: 'Open an approved HTTPS URL in the system browser. Security: allowlist-only domains.',
    category: 'Browser & Web',
    outputType: 'Action',
    steps: 1,
  },
  {
    name: 'web_fetch',
    displayName: 'Fetch Web Content',
    description: 'Fetch and extract content from web pages. Handles HTML parsing and text extraction.',
    category: 'Browser & Web',
    outputType: 'Output',
    steps: 1,
  },
  {
    name: 'web_search',
    displayName: 'Web Search',
    description: 'Search the web for information. Returns relevant search results with titles, URLs, and descriptions.',
    category: 'Browser & Web',
    outputType: 'Query',
    steps: 1,
  },
  {
    name: 'http_request',
    displayName: 'HTTP Request',
    description: 'Make HTTP requests to external APIs. Supports GET, POST, PUT, DELETE, PATCH methods.',
    category: 'Browser & Web',
    outputType: 'Action',
    steps: 1,
  },

  // Scheduling
  {
    name: 'schedule',
    displayName: 'Schedule Task',
    description: 'Manage scheduled shell-only tasks. Actions: create, add, once, list, get, cancel, remove.',
    category: 'Scheduling',
    outputType: 'Action',
    steps: 2,
  },
  {
    name: 'cron_add',
    displayName: 'Add Cron Job',
    description: 'Create a new scheduled cron job with flexible timing options and job types.',
    category: 'Scheduling',
    outputType: 'Action',
    steps: 1,
  },
  {
    name: 'cron_list',
    displayName: 'List Cron Jobs',
    description: 'List all scheduled cron jobs with their status and next run times.',
    category: 'Scheduling',
    outputType: 'Query',
    steps: 1,
  },
  {
    name: 'cron_remove',
    displayName: 'Remove Cron Job',
    description: 'Remove a cron job by id. Permanently deletes the scheduled task.',
    category: 'Scheduling',
    outputType: 'Action',
    steps: 1,
  },
  {
    name: 'cron_update',
    displayName: 'Update Cron Job',
    description: 'Modify an existing cron job schedule or configuration.',
    category: 'Scheduling',
    outputType: 'Action',
    steps: 1,
  },
  {
    name: 'cron_run',
    displayName: 'Run Cron Job Now',
    description: 'Manually trigger a cron job to run immediately, outside its schedule.',
    category: 'Scheduling',
    outputType: 'Action',
    steps: 1,
  },
  {
    name: 'cron_runs',
    displayName: 'Cron Run History',
    description: 'List recent run history for a cron job including status and output.',
    category: 'Scheduling',
    outputType: 'Query',
    steps: 1,
  },

  // Git & Version Control
  {
    name: 'git_operations',
    displayName: 'Git Operations',
    description: 'Perform structured Git operations: status, diff, log, branch, commit, add, checkout, stash.',
    category: 'Git',
    outputType: 'Action',
    steps: 1,
  },

  // Integrations
  {
    name: 'composio',
    displayName: 'Composio Actions',
    description: 'Execute actions on 1000+ apps via Composio (Gmail, Notion, GitHub, Slack, etc.).',
    category: 'Integrations',
    outputType: 'Action',
    steps: 2,
  },
  {
    name: 'one',
    displayName: 'One CLI Actions',
    description: 'Execute actions on 200+ third-party platforms (Gmail, Slack, GitHub) through the One CLI.',
    category: 'Integrations',
    outputType: 'Action',
    steps: 2,
  },
  {
    name: 'pushover',
    displayName: 'Pushover Notification',
    description: 'Send push notifications via Pushover to mobile devices and desktop.',
    category: 'Integrations',
    outputType: 'Action',
    steps: 1,
  },
  {
    name: 'delegate',
    displayName: 'Delegate to Sub-Agent',
    description: 'Delegate a task to a specialized sub-agent with its own model and capabilities.',
    category: 'Integrations',
    outputType: 'Draft',
    steps: 3,
  },

  // Configuration
  {
    name: 'model_routing_config',
    displayName: 'Model Routing Config',
    description: 'Manage default model settings, scenario-based routes, classification rules, and delegate profiles.',
    category: 'Configuration',
    outputType: 'Action',
    steps: 1,
  },
  {
    name: 'proxy_config',
    displayName: 'Proxy Configuration',
    description: 'Configure proxy settings for outbound requests and API calls.',
    category: 'Configuration',
    outputType: 'Action',
    steps: 1,
  },

  // SOPs
  {
    name: 'sop_list',
    displayName: 'List SOPs',
    description: 'List all loaded Standard Operating Procedures with triggers, priority, and step count.',
    category: 'SOPs',
    outputType: 'Query',
    steps: 1,
  },
  {
    name: 'sop_execute',
    displayName: 'Execute SOP',
    description: 'Start execution of a Standard Operating Procedure by name or trigger match.',
    category: 'SOPs',
    outputType: 'Action',
    steps: 2,
  },
  {
    name: 'sop_status',
    displayName: 'SOP Status',
    description: 'Query SOP execution status. Provide run_id for specific run or sop_name for all runs.',
    category: 'SOPs',
    outputType: 'Query',
    steps: 1,
  },
  {
    name: 'sop_advance',
    displayName: 'Advance SOP Step',
    description: 'Report the result of the current SOP step and advance to the next step.',
    category: 'SOPs',
    outputType: 'Action',
    steps: 1,
  },
  {
    name: 'sop_approve',
    displayName: 'Approve SOP Step',
    description: 'Approve a pending SOP step that is waiting for operator approval.',
    category: 'SOPs',
    outputType: 'Action',
    steps: 1,
  },

  // Hardware (optional)
  {
    name: 'hardware_board_info',
    displayName: 'Hardware Board Info',
    description: 'Return full board info (chip, architecture, memory map) for connected hardware.',
    category: 'Hardware',
    outputType: 'Query',
    steps: 1,
  },
  {
    name: 'hardware_memory_map',
    displayName: 'Hardware Memory Map',
    description: 'Return the memory map (flash and RAM address ranges) for connected hardware.',
    category: 'Hardware',
    outputType: 'Query',
    steps: 1,
  },
  {
    name: 'hardware_memory_read',
    displayName: 'Read Hardware Memory',
    description: 'Read actual memory/register values from Nucleo via USB. Returns hex dump.',
    category: 'Hardware',
    outputType: 'Output',
    steps: 1,
  },
])

const categories = computed(() => {
  const cats = new Set(tools.value.map(t => t.category))
  return ['All', ...Array.from(cats).sort()]
})

const activeCategory = ref('All')
const searchQuery = ref('')

const filteredTools = computed(() => {
  let result = tools.value

  if (activeCategory.value !== 'All') {
    result = result.filter(t => t.category === activeCategory.value)
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(t =>
      t.displayName.toLowerCase().includes(q) ||
      t.description.toLowerCase().includes(q) ||
      t.name.toLowerCase().includes(q)
    )
  }

  return result
})

const groupedTools = computed(() => {
  const groups: Record<string, ToolDefinition[]> = {}

  // First add favorites if on "All" tab
  if (activeCategory.value === 'All') {
    const favorites = filteredTools.value.filter(t => t.favorite)
    if (favorites.length > 0) {
      groups['Recommended for You'] = favorites
    }
  }

  // Group by category
  for (const tool of filteredTools.value) {
    if (!groups[tool.category]) {
      groups[tool.category] = []
    }
    const currentGroup = groups[tool.category] as ToolDefinition[]
    const recommendedGroup = groups['Recommended for You']
    if (!tool.favorite || activeCategory.value !== 'All') {
      currentGroup.push(tool)
    } else if (!recommendedGroup?.includes(tool)) {
      currentGroup.push(tool)
    }
  }

  // Remove empty groups
  for (const key of Object.keys(groups)) {
    const group = groups[key]
    if (group && group.length === 0) {
      delete groups[key]
    }
  }

  return groups
})

function toggleFavorite(tool: ToolDefinition) {
  tool.favorite = !tool.favorite
}

function outputIcon(type: string): string {
  switch (type) {
    case 'Output': return 'ph:file-text'
    case 'Draft': return 'ph:pencil-simple'
    case 'Action': return 'ph:cursor-click'
    case 'Query': return 'ph:magnifying-glass'
    default: return 'ph:cube'
  }
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
          placeholder="Search tools..."
          class="w-full pl-10 pr-4 py-2 text-[13px] bg-card/50 border border-border/50 rounded-xl focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground"
        />
      </div>

      <!-- Category tabs -->
      <div class="flex items-center gap-1 overflow-x-auto pb-1">
        <button
          v-for="cat in categories"
          :key="cat"
          @click="activeCategory = cat"
          class="px-3 py-1.5 text-[12px] font-medium rounded-lg whitespace-nowrap transition-colors"
          :class="activeCategory === cat
            ? 'bg-foreground text-background'
            : 'text-muted-foreground hover:text-foreground hover:bg-card/50'"
        >
          {{ cat }}
        </button>
      </div>
    </div>

    <!-- Tools grid -->
    <div class="flex-1 overflow-y-auto px-6 py-4">
      <div v-if="Object.keys(groupedTools).length === 0" class="flex items-center justify-center h-full">
        <p class="text-muted-foreground text-[12px]">No tools found matching your search.</p>
      </div>

      <div v-else class="space-y-6">
        <section v-for="(toolList, groupName) in groupedTools" :key="groupName">
          <h2 class="font-sans text-[20px] font-medium text-foreground tracking-tight mb-3">{{ groupName }}</h2>
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
            <div
              v-for="tool in toolList"
              :key="tool.name"
              class="group relative p-4 rounded-2xl border border-border/40 bg-card/20 hover:bg-card/40 hover:border-border/60 transition-all cursor-pointer"
            >
              <!-- Favorite star -->
              <button
                @click.stop="toggleFavorite(tool)"
                class="absolute top-3 right-3 text-muted-foreground hover:text-amber-500 transition-colors"
                :class="{ 'text-amber-500': tool.favorite }"
              >
                <Icon :icon="tool.favorite ? 'ph:star-fill' : 'ph:star'" class="size-4" />
              </button>

              <!-- Title -->
              <h3 class="font-sans text-[15px] font-medium text-foreground pr-6 mb-1">
                {{ tool.displayName }}
              </h3>

              <!-- Description -->
              <p class="text-[13px] text-muted-foreground line-clamp-2 mb-3">
                {{ tool.description }}
              </p>

              <!-- Meta row -->
              <div class="flex items-center gap-3 text-[12px] text-muted-foreground">
                <span class="flex items-center gap-1">
                  <Icon :icon="outputIcon(tool.outputType)" class="size-3.5" />
                  {{ tool.outputType }}
                </span>
                <span v-if="tool.steps" class="flex items-center gap-1">
                  ·
                  {{ tool.steps }} {{ tool.steps === 1 ? 'step' : 'steps' }}
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
