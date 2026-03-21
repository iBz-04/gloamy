<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { ref } from 'vue'
import ModelSelector from '@/components/ModelSelector.vue'
</script>

<template>
  <header
    class="h-12 flex items-center justify-between px-4 shrink-0 select-none"
    data-tauri-drag-region
  >
    <div class="flex items-center gap-2">
      <ModelSelector />
    </div>

    <div class="flex items-center gap-3">
      <button class="text-muted-foreground hover:text-foreground transition-colors relative">
        <Icon icon="ph:bell" class="size-[18px]" />
      </button>

      <div class="flex items-center gap-1 text-xs text-muted-foreground">
        <Icon icon="ph:lightning" class="size-3.5" />
        <span class="font-medium">8720</span>
      </div>

      <button class="size-7 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-xs font-semibold">
        G
      </button>
    </div>
  </header>
</template>
