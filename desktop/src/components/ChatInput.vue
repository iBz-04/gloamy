<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { ref } from 'vue'
import FolderSelector from '@/components/FolderSelector.vue'

const message = ref('')

function submit() {
  if (!message.value.trim()) return
  message.value = ''
}
</script>

<template>
  <div class="w-full max-w-[680px] mx-auto">
    <div class="bg-card border border-border rounded-xl shadow-sm overflow-hidden">
      <div class="px-4 py-3">
        <input
          v-model="message"
          type="text"
          placeholder="Give Gloamy a task to work on..."
          class="w-full bg-transparent text-sm text-foreground placeholder:text-muted-foreground/60 outline-none"
          @keydown.enter="submit"
        />
      </div>

      <div class="flex items-center justify-between px-3 pb-3">
        <div class="flex items-center gap-1.5">
          <button class="p-1.5 text-muted-foreground hover:text-foreground hover:bg-muted/30 rounded-md transition-colors">
            <Icon icon="ph:plus" class="size-4" />
          </button>
          <button class="p-1.5 text-muted-foreground hover:text-foreground hover:bg-muted/30 rounded-md transition-colors">
            <Icon icon="ph:wrench" class="size-4" />
          </button>
          <FolderSelector />
        </div>

        <div class="flex items-center gap-2">
          <button class="p-1.5 text-muted-foreground hover:text-foreground transition-colors">
            <Icon icon="ph:microphone" class="size-4" />
          </button>

          <button
            class="size-8 rounded-full flex items-center justify-center transition-colors"
            :class="message.trim() ? 'bg-primary text-primary-foreground' : 'bg-muted/50 text-muted-foreground'"
            @click="submit"
          >
            <Icon icon="ph:arrow-up-bold" class="size-4" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
