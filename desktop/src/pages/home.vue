<script setup lang="ts">
import ChatInput from '@/components/ChatInput.vue'
</script>

<template>
  <div class="h-full w-full flex flex-col items-center justify-center px-6">
    <h1
      v-motion
      :initial="{ opacity: 0, y: 20 }"
      :enter="{ opacity: 1, y: 0, transition: { duration: 400, delay: 100 } }"
      class="text-3xl font-semibold text-foreground mb-10 font-display"
    >
      What can I do for you?
    </h1>

    <div
      v-motion
      :initial="{ opacity: 0, y: 16 }"
      :enter="{ opacity: 1, y: 0, transition: { duration: 400, delay: 200 } }"
      class="w-full"
    >
      <ChatInput />
    </div>
  </div>
</template>
