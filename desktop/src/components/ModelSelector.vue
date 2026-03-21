<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { ref } from 'vue'

const open = ref(false)
const selectedModel = ref('Gloamy 1.0 Max')

const models = [
  { label: 'Gloamy 1.0 Max', description: 'Most capable' },
  { label: 'Gloamy 1.0 Mini', description: 'Fast & efficient' },
]

function selectModel(model: typeof models[0]) {
  selectedModel.value = model.label
  open.value = false
}
</script>

<template>
  <div class="relative">
    <button
      class="flex items-center gap-1.5 px-2 py-1 text-sm font-medium text-foreground hover:bg-muted/30 rounded-md transition-colors"
      @click="open = !open"
    >
      {{ selectedModel }}
      <Icon icon="ph:caret-down" class="size-3 text-muted-foreground" />
    </button>

    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="opacity-0 scale-95 -translate-y-1"
      leave-active-class="transition duration-100 ease-in"
      leave-to-class="opacity-0 scale-95 -translate-y-1"
    >
      <div
        v-if="open"
        class="absolute top-full left-0 mt-1 w-52 bg-popover border border-border rounded-lg shadow-md py-1 z-50"
      >
        <button
          v-for="model in models"
          :key="model.label"
          class="w-full flex items-center justify-between px-3 py-2 text-sm hover:bg-muted/30 transition-colors"
          @click="selectModel(model)"
        >
          <div class="text-left">
            <div class="font-medium text-foreground">{{ model.label }}</div>
            <div class="text-[11px] text-muted-foreground">{{ model.description }}</div>
          </div>
          <Icon v-if="selectedModel === model.label" icon="ph:check" class="size-4 text-foreground" />
        </button>
      </div>
    </Transition>

    <div v-if="open" class="fixed inset-0 z-40" @click="open = false" />
  </div>
</template>
