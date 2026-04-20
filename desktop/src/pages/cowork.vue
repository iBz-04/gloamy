<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { ref, nextTick, unref } from 'vue'
import CoworkInput from '@/components/CoworkInput.vue'
import { useAuthStore } from '@/stores/auth'
import { SSEClient, type SSEEvent } from '@/lib/sse'

interface StreamStep {
  id: string
  text: string
}

interface Message {
  id: string
  role: 'user' | 'assistant'
  content: string
  status?: 'pending' | 'ready' | 'error'
  steps?: StreamStep[]
  elapsedMs?: number
  collapsed?: boolean
}

interface ApiChatResponse {
  response?: unknown
  model?: unknown
}

const auth = useAuthStore()
const messages = ref<Message[]>([])
const chatContainer = ref<HTMLElement | null>(null)
const isSubmitting = ref(false)
let messageCounter = 0
let stepCounter = 0

function humanizeTool(tool: string): string {
  const map: Record<string, string> = {
    shell: 'Running shell command',
    file: 'Reading workspace files',
    browser: 'Browsing the web',
    memory: 'Consulting memory',
    mac_automation: 'Automating macOS',
    search: 'Searching',
  }
  if (map[tool]) return map[tool]
  return `Using ${tool.replace(/[_-]/g, ' ')}`
}

const savedPrompts = [
  {
    title: 'Turn website or blog post into a slide deck',
    icons: ['hugeicons:presentation-02']
  },
  {
    title: 'Research a topic in depth (up to 10 minutes)',
    icons: ['hugeicons:search-02']
  },
  {
    title: 'Generate an image based on a description',
    icons: ['hugeicons:image-01']
  },
  {
    title: 'Research & present a topic with slides',
    icons: ['hugeicons:book-open-01']
  },
  {
    title: 'Personalize sales deck for target customer',
    icons: ['hugeicons:user-group']
  },
  {
    title: 'Compare product options with deep research',
    icons: ['hugeicons:analytics-01']
  }
]

async function scrollToBottom() {
  await nextTick()
  if (chatContainer.value) {
    chatContainer.value.scrollTop = chatContainer.value.scrollHeight
  }
}

function nextMessageId(): string {
  messageCounter += 1
  return `cowork-${messageCounter}`
}

function normalizeAssistantResponse(payload: ApiChatResponse): string {
  if (typeof payload.response === 'string' && payload.response.trim()) {
    return payload.response
  }
  return 'The agent completed the request but returned an empty response.'
}

async function handleUserSubmit(text: string) {
  if (isSubmitting.value) {
    return
  }

  messages.value.push({
    id: nextMessageId(),
    role: 'user',
    content: text,
    status: 'ready',
  })

  const placeholderId = nextMessageId()
  const startedAt = performance.now()
  const placeholder: Message = {
    id: placeholderId,
    role: 'assistant',
    content: '',
    status: 'pending',
    steps: [],
    collapsed: false,
  }
  messages.value.push(placeholder)

  const appendStep = (textLine: string) => {
    const msg = messages.value.find(m => m.id === placeholderId)
    if (!msg || msg.status !== 'pending') return
    stepCounter += 1
    msg.steps = msg.steps ?? []
    const last = msg.steps[msg.steps.length - 1]
    if (last && last.text === textLine) return
    msg.steps.push({ id: `step-${stepCounter}`, text: textLine })
    scrollToBottom()
  }

  const sse = new SSEClient({
    path: '/api/events',
    getBaseUrl: () => String(unref(auth.baseUrl) ?? '').trim(),
    getToken: () => {
      const t = unref(auth.token)
      return typeof t === 'string' ? t : null
    },
    autoReconnect: false,
  })
  sse.onEvent = (evt: SSEEvent) => {
    switch (evt.type) {
      case 'agent_start':
        appendStep('Thinking through the request')
        break
      case 'tool_call_start':
        if (typeof evt.tool === 'string') appendStep(humanizeTool(evt.tool))
        break
      case 'error':
        if (typeof evt.message === 'string') appendStep(`Error: ${evt.message}`)
        break
    }
  }
  sse.connect()

  isSubmitting.value = true
  await scrollToBottom()

  try {
    const response = await auth.fetchWithAuth<ApiChatResponse>('/api/chat', {
      method: 'POST',
      body: JSON.stringify({ message: text }),
    })

    const assistantMessage = messages.value.find(message => message.id === placeholderId)
    if (assistantMessage) {
      assistantMessage.content = normalizeAssistantResponse(response)
      assistantMessage.status = 'ready'
      assistantMessage.elapsedMs = performance.now() - startedAt
      assistantMessage.collapsed = true
    }
  }
  catch (error: unknown) {
    const assistantMessage = messages.value.find(message => message.id === placeholderId)
    if (assistantMessage) {
      assistantMessage.content = error instanceof Error ? error.message : 'Failed to reach the desktop agent.'
      assistantMessage.status = 'error'
      assistantMessage.elapsedMs = performance.now() - startedAt
      assistantMessage.collapsed = true
    }
  }
  finally {
    sse.disconnect()
    isSubmitting.value = false
    await scrollToBottom()
  }
}

function toggleSteps(msg: Message) {
  msg.collapsed = !msg.collapsed
}

function formatElapsed(ms?: number): string {
  if (!ms || ms < 0) return ''
  const s = Math.max(1, Math.round(ms / 1000))
  if (s < 60) return `${s}s`
  const m = Math.floor(s / 60)
  const rem = s % 60
  return rem ? `${m}m ${rem}s` : `${m}m`
}
</script>

<template>
  <div class="h-full w-full flex flex-col bg-background font-sans relative">
    
    <!-- Empty State (Landing Page) -->
    <div v-if="messages.length === 0" class="flex-1 flex flex-col items-center justify-center px-6 pb-32 select-none overflow-y-auto">
      <div class="w-full max-w-[720px] flex flex-col items-center">
        <!-- Integrated Input Card with Mascot peeking from behind -->
        <div class="w-full relative mt-12 z-0">
          <div class="absolute bottom-full left-1/2 -translate-x-1/2 w-16 translate-y-3 -z-10">
            <img src="/gloamyowl.png" alt="Gloamy Mascot" class="w-full h-auto block" />
          </div>
          <CoworkInput class="w-full relative z-10" :disabled="isSubmitting" @submit="handleUserSubmit" />
        </div>

        <!-- Saved Prompts Section -->
        <div class="w-full mt-10">
          <div class="flex items-center justify-between mb-4 px-1">
            <h2 class="text-[14px] font-medium text-foreground/90">Saved prompts</h2>
            <button class="flex items-center gap-1 text-[13px] text-muted-foreground hover:text-foreground/90 transition-colors">
              View all
              <Icon icon="hugeicons:arrow-right-01-sharp" class="size-3" />
            </button>
          </div>

          <div class="grid grid-cols-3 gap-3">
            <div
              v-for="(prompt, i) in savedPrompts"
              :key="i"
              class="bg-card/40 hover:bg-card/80 border border-transparent hover:border-border/50 transition-all rounded-[16px] p-4 cursor-pointer flex flex-col min-h-[110px]"
              @click="handleUserSubmit(prompt.title)"
            >
              <div class="flex items-center gap-1.5 mb-3">
                <Icon
                  v-for="(icon, idx) in prompt.icons"
                  :key="idx"
                  :icon="icon"
                  class="size-5 text-muted-foreground"
                />
              </div>
              <p class="text-[13px] font-medium text-foreground/90 leading-[1.4]">
                {{ prompt.title }}
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Chat State -->
    <div v-else class="flex-1 flex flex-col overflow-hidden relative">
      <!-- Message List -->
      <div ref="chatContainer" class="flex-1 overflow-y-auto px-6 pt-8 pb-40">
        <div class="w-full max-w-[760px] mx-auto flex flex-col gap-6">
          <div 
            v-for="msg in messages" 
            :key="msg.id" 
            class="w-full flex" 
            :class="msg.role === 'user' ? 'justify-end' : 'justify-start'"
          >
            <!-- User Message -->
            <div v-if="msg.role === 'user'" class="bg-muted/50 px-4 py-2.5 rounded-[20px] max-w-[85%] text-[15px] text-foreground">
              {{ msg.content }}
            </div>

            <!-- Assistant Message -->
            <div v-else class="max-w-[100%] w-full flex flex-col gap-2">
              <!-- Streaming steps card (live or collapsed summary) -->
              <div v-if="(msg.steps?.length ?? 0) > 0 || msg.status === 'pending'" class="w-full">
                <!-- Collapsed summary header (after completion) -->
                <button
                  v-if="msg.status !== 'pending' && msg.collapsed"
                  class="flex items-center gap-1 text-[13.5px] text-muted-foreground hover:text-foreground/80 transition-colors mb-1"
                  @click="toggleSteps(msg)"
                >
                  <span>Thought{{ msg.elapsedMs ? ` for ${formatElapsed(msg.elapsedMs)}` : '' }}</span>
                  <Icon icon="hugeicons:arrow-right-01-sharp" class="size-3" />
                </button>

                <!-- Expanded card with steps -->
                <div
                  v-else
                  class="border border-border/60 rounded-[14px] overflow-hidden"
                >
                  <div class="flex items-center justify-between px-4 py-2.5 border-b border-border/50">
                    <div class="flex items-center gap-2">
                      <Icon
                        v-if="msg.status === 'pending'"
                        icon="svg-spinners:3-dots-fade"
                        class="size-4 text-muted-foreground"
                      />
                      <Icon
                        v-else
                        icon="hugeicons:checkmark-circle-02"
                        class="size-4 text-muted-foreground"
                      />
                      <span class="text-[13.5px] font-semibold text-foreground">
                        {{ msg.status === 'pending' ? 'Working' : `Thought${msg.elapsedMs ? ` for ${formatElapsed(msg.elapsedMs)}` : ''}` }}
                      </span>
                    </div>
                    <button
                      v-if="msg.status !== 'pending'"
                      class="text-muted-foreground hover:text-foreground/80 transition-colors"
                      @click="toggleSteps(msg)"
                    >
                      <Icon icon="hugeicons:arrow-up-01-sharp" class="size-4" />
                    </button>
                  </div>
                  <div class="relative px-4 py-3">
                    <div class="absolute left-[22px] top-3 bottom-3 w-px bg-border/60"></div>
                    <div class="flex flex-col gap-2.5">
                      <div
                        v-for="step in msg.steps"
                        :key="step.id"
                        class="flex items-start gap-3 relative"
                      >
                        <Icon icon="hugeicons:cursor-02" class="size-4 mt-0.5 text-muted-foreground shrink-0 relative z-10 bg-background" />
                        <div class="text-[13.5px] leading-[1.5] text-foreground/85">
                          {{ step.text }}
                        </div>
                      </div>
                      <div
                        v-if="msg.status === 'pending' && (msg.steps?.length ?? 0) === 0"
                        class="flex items-center gap-3"
                      >
                        <Icon icon="svg-spinners:3-dots-fade" class="size-4 text-muted-foreground" />
                        <span class="text-[13.5px] text-muted-foreground">Getting started…</span>
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <div
                v-if="msg.content"
                class="text-[15px] leading-[1.6] whitespace-pre-wrap"
                :class="msg.status === 'error' ? 'text-destructive' : 'text-foreground'"
              >
                {{ msg.content }}
              </div>

              <div v-if="msg.status !== 'pending'" class="flex items-center gap-3 mt-1.5 text-muted-foreground/60">
                <button class="hover:text-foreground/80 transition-colors"><Icon icon="hugeicons:copy-01" class="size-[16px]" /></button>
                <button class="hover:text-foreground/80 transition-colors"><Icon icon="hugeicons:thumbs-up" class="size-[16px]" /></button>
                <button class="hover:text-foreground/80 transition-colors"><Icon icon="hugeicons:thumbs-down" class="size-[16px]" /></button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Sticky Input Area -->
      <div class="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-background via-background to-transparent pt-10 pb-6 px-6">
        <div class="w-full max-w-[760px] mx-auto">
          <CoworkInput class="w-full" :disabled="isSubmitting" @submit="handleUserSubmit" />
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped>
/* Scoped styles */
</style>
