<script setup lang="ts">
import { Icon } from '@iconify/vue'
import CoworkInput from '@/components/CoworkInput.vue'

const savedPrompts = [
  {
    title: 'Turn website or blog post into a slide deck',
    icons: ['hugeicons:presentation-02']
  },
  {
    title: 'Research a topic in depth (up to 30 minutes)',
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
</script>

<template>
  <div class="h-full w-full flex flex-col items-center justify-center px-6 pb-32 bg-background select-none font-sans">
    <div class="w-full max-w-[720px] flex flex-col items-center">
      <!-- Title -->
      <h1 class="text-[32px] font-medium tracking-tight text-foreground mb-8">
        Lets get to work!
      </h1>

      <!-- Integrated Input Card -->
      <div class="w-full flex flex-col items-center">
        <CoworkInput />
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
          >
            <!-- Icons -->
            <div class="flex items-center gap-1.5 mb-3">
              <Icon
                v-for="(icon, idx) in prompt.icons"
                :key="idx"
                :icon="icon"
                class="size-5 text-muted-foreground"
              />
            </div>
            
            <!-- Text -->
            <p class="text-[13px] font-medium text-foreground/90 leading-[1.4]">
              {{ prompt.title }}
            </p>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>

<style scoped>
/* Any specific page styling if needed */
</style>
