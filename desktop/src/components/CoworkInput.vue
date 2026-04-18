<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { Icon } from '@iconify/vue'

const message = ref('')
const textarea = ref<HTMLTextAreaElement | null>(null)

function adjustHeight() {
  if (!textarea.value) return
  textarea.value.style.height = 'auto'
  textarea.value.style.height = `${textarea.value.scrollHeight}px`
}

watch(message, () => {
  adjustHeight()
})

onMounted(() => {
  adjustHeight()
})

function submit() {
  if (!message.value.trim()) return
  console.log('Submitting:', message.value)
  message.value = ''
  if (textarea.value) textarea.value.style.height = 'auto'
}
</script>

<template>
  <div class="w-full">
    <div class="flex flex-col bg-card border border-border/50 rounded-[20px] overflow-hidden shadow-[0_2px_12px_rgba(0,0,0,0.03)] dark:shadow-none transition-all duration-300 focus-within:shadow-[0_4px_24px_rgba(0,0,0,0.06)] dark:focus-within:shadow-none focus-within:border-border">
      
      <!-- Top Input Area -->
      <div class="px-4 pt-4 pb-3">
        <textarea
          ref="textarea"
          v-model="message"
          rows="1"
          placeholder="Ask gloamy to complete a task for you"
          class="w-full bg-transparent text-[15px] text-foreground placeholder:text-muted-foreground/60 outline-none resize-none min-h-[44px] max-h-[300px] leading-relaxed transition-[height] duration-200"
          @keydown.enter.prevent="submit"
        />
        
        <!-- Action Bar -->
        <div class="flex items-center justify-between mt-2">
          <div class="flex items-center gap-4 text-[13px] text-muted-foreground font-medium">
            <button class="hover:text-foreground transition-colors">
              <Icon icon="hugeicons:add-01" class="size-4" />
            </button>
            <button class="flex items-center gap-1 hover:text-foreground transition-colors">
              <Icon icon="hugeicons:hand-02" class="size-4 opacity-70" />
              <Icon icon="hugeicons:arrow-down-01-sharp" class="size-3 opacity-50" />
            </button>
          </div>
          
          <div class="flex items-center gap-4 text-[13px] text-muted-foreground font-medium">
            <button class="hover:text-foreground transition-colors">
              <Icon icon="hugeicons:mic-01" class="size-4" />
            </button>
            <button
              class="size-8 rounded-full flex items-center justify-center transition-all duration-300 ml-1"
              :class="message.trim() ? 'bg-foreground text-background shadow-md dark:shadow-none scale-100 hover:opacity-80' : 'bg-muted/50 text-muted-foreground/40 scale-95 cursor-not-allowed'"
              :disabled="!message.trim()"
              @click="submit"
            >
              <Icon icon="hugeicons:sent" class="size-4" />
            </button>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>
