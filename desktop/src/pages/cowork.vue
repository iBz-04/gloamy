<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { ref, nextTick } from 'vue'
import CoworkInput from '@/components/CoworkInput.vue'
import { useAuthStore } from '@/stores/auth'

interface Message {
  id: string
  role: 'user' | 'assistant'
  content: string
  status?: 'pending' | 'ready' | 'error'
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
  messages.value.push({
    id: placeholderId,
    role: 'assistant',
    content: 'Working on it...',
    status: 'pending',
  })

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
    }
  }
  catch (error: unknown) {
    const assistantMessage = messages.value.find(message => message.id === placeholderId)
    if (assistantMessage) {
      assistantMessage.content = error instanceof Error ? error.message : 'Failed to reach the desktop agent.'
      assistantMessage.status = 'error'
    }
  }
  finally {
    isSubmitting.value = false
    await scrollToBottom()
  }
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
              <div
                v-if="msg.status === 'pending'"
                class="text-[13.5px] text-muted-foreground font-medium"
              >
                Thinking...
              </div>

              <div
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
