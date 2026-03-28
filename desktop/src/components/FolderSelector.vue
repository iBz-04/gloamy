<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { ref } from 'vue'

const open = ref(false)
const selectedFolder = ref({ name: 'Data Analysis', path: '/Users/Gloamy/Desktop/Data-Analysis' })

const folders = [
  { name: 'Data Analysis', path: '/Users/Gloamy/Desktop/Data-Analysis' },
  { name: 'Work', path: '/Users/Gloamy/Desktop/Work' },
]

function selectFolder(folder: typeof folders[0]) {
  selectedFolder.value = folder
  open.value = false
}
</script>

<template>
  <div class="relative">
    <button
      class="flex items-center gap-1.5 px-2.5 py-1 text-[13px] text-muted-foreground hover:text-foreground bg-muted/30 hover:bg-muted/50 rounded-md transition-colors"
      @click="open = !open"
    >
      <Icon icon="hugeicons:folder-01" class="size-3.5" />
      {{ selectedFolder.name }}
    </button>

    <Transition
      enter-active-class="transition duration-150 ease-out"
      enter-from-class="opacity-0 scale-95 translate-y-1"
      leave-active-class="transition duration-100 ease-in"
      leave-to-class="opacity-0 scale-95 translate-y-1"
    >
      <div
        v-if="open"
        class="absolute bottom-full left-0 mb-2 w-64 bg-popover border border-border rounded-lg shadow-lg py-1.5 z-50"
      >
        <div class="px-3 py-1.5 text-[11px] font-medium text-muted-foreground uppercase tracking-wider">
          My Computer
        </div>

        <button
          v-for="folder in folders"
          :key="folder.name"
          class="w-full flex items-center justify-between px-3 py-2 hover:bg-muted/30 transition-colors"
          @click="selectFolder(folder)"
        >
          <div class="flex items-center gap-2.5">
            <Icon icon="hugeicons:folder-01" class="size-4 text-muted-foreground" />
            <div class="text-left">
              <div class="text-[13px] font-medium text-foreground">{{ folder.name }}</div>
              <div class="text-[11px] text-muted-foreground">{{ folder.path }}</div>
            </div>
          </div>
          <Icon v-if="selectedFolder.name === folder.name" icon="hugeicons:tick-02" class="size-4 text-foreground" />
        </button>

        <div class="border-t border-border mt-1 pt-1">
          <button class="w-full flex items-center gap-2.5 px-3 py-2 text-[13px] text-muted-foreground hover:text-foreground hover:bg-muted/30 transition-colors">
            <Icon icon="hugeicons:add-01" class="size-4" />
            Add local folder
          </button>
          <button class="w-full flex items-center gap-2.5 px-3 py-2 text-[13px] text-muted-foreground hover:text-foreground hover:bg-muted/30 transition-colors">
            <Icon icon="hugeicons:settings-01" class="size-4" />
            Manage folders
          </button>
        </div>
      </div>
    </Transition>

    <div v-if="open" class="fixed inset-0 z-40" @click="open = false" />
  </div>
</template>
