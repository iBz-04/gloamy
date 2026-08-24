<script setup lang="ts">
import { computed } from 'vue'
import { cn } from '@/lib/utils'

type SwitchSize = 'sm' | 'md'

const props = withDefaults(
  defineProps<{
    modelValue: boolean
    disabled?: boolean
    size?: SwitchSize
    class?: string
  }>(),
  {
    disabled: false,
    size: 'md',
  },
)

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void
}>()

function toggle() {
  if (props.disabled) return
  emit('update:modelValue', !props.modelValue)
}

function onKeydown(e: KeyboardEvent) {
  if (e.key !== ' ' && e.key !== 'Enter') return
  e.preventDefault()
  toggle()
}

const trackSizeClasses = computed(() => (props.size === 'sm' ? 'h-4 w-8' : 'h-5 w-9'))
const thumbSizeClasses = computed(() => (props.size === 'sm' ? 'top-0.5 left-0.5 size-3' : 'top-0.5 left-0.5 size-4'))
</script>

<template>
  <button
    type="button"
    role="switch"
    :aria-checked="modelValue"
    :aria-disabled="disabled || undefined"
    :disabled="disabled"
    :class="cn(
      'relative shrink-0 cursor-pointer rounded-[2px] transition-colors focus-visible:outline-none focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50',
      trackSizeClasses,
      modelValue
        ? 'bg-foreground/70 hover:bg-foreground/75'
        : 'bg-muted/70 hover:bg-muted/90',
      props.class,
    )"
    @click="toggle"
    @keydown="onKeydown"
  >
    <span
      :class="cn(
        'absolute rounded-[2px] bg-background shadow-sm transition-transform',
        thumbSizeClasses,
        modelValue ? 'translate-x-4' : 'translate-x-0',
      )"
    />
  </button>
</template>
