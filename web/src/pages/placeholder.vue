<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

const title = computed(() => (route.meta.title as string) ?? 'Coming soon')
const description = computed(() => (route.meta.description as string) ?? 'This section is not available yet.')
</script>

<template>
  <div class="h-full w-full flex items-center justify-center px-6 text-center">
    <div class="space-y-2">
      <h1 class="text-2xl font-semibold text-foreground">{{ title }}</h1>
      <p class="text-muted-foreground max-w-md mx-auto text-sm">
        {{ description }}
      </p>
    </div>
  </div>
</template>
