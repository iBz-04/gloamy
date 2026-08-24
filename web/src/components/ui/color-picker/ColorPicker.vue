<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted, onBeforeUnmount } from 'vue'

const props = defineProps<{
  modelValue: string
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()

const isOpen = ref(false)
const pickerRef = ref<HTMLDivElement>()
const canvasRef = ref<HTMLCanvasElement>()
const hueRef = ref<HTMLCanvasElement>()

function hsvToHex(h: number, s: number, v: number): string {
  s /= 100
  v /= 100
  const f = (n: number) => {
    const k = (n + h / 60) % 6
    return Math.round(255 * (v - v * s * Math.max(Math.min(k, 4 - k, 1), 0)))
      .toString(16).padStart(2, '0')
  }
  return `#${f(5)}${f(3)}${f(1)}`
}

function hexToHsv(hex: string): { h: number; s: number; v: number } {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  if (!result) return { h: 0, s: 100, v: 100 }
  
  const r = parseInt(result[1]!, 16) / 255
  const g = parseInt(result[2]!, 16) / 255
  const b = parseInt(result[3]!, 16) / 255

  const max = Math.max(r, g, b)
  const min = Math.min(r, g, b)
  const d = max - min
  
  let h = 0
  const s = max === 0 ? 0 : (d / max) * 100
  const v = max * 100

  if (max !== min) {
    switch (max) {
      case r: h = ((g - b) / d + (g < b ? 6 : 0)) * 60; break
      case g: h = ((b - r) / d + 2) * 60; break
      case b: h = ((r - g) / d + 4) * 60; break
    }
  }

  return { h: Math.round(h), s: Math.round(s), v: Math.round(v) }
}

const currentHsv = ref(hexToHsv(props.modelValue))

watch(() => props.modelValue, (newVal) => {
  currentHsv.value = hexToHsv(newVal)
})

const displayHex = computed(() => props.modelValue.replace('#', '').toUpperCase())

// Draw the saturation/value gradient
function drawSaturationGradient() {
  const canvas = canvasRef.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const width = canvas.width
  const height = canvas.height

  // Create horizontal white gradient
  const whiteGradient = ctx.createLinearGradient(0, 0, width, 0)
  whiteGradient.addColorStop(0, '#ffffff')
  whiteGradient.addColorStop(1, hsvToHex(currentHsv.value.h, 100, 100))
  
  ctx.fillStyle = whiteGradient
  ctx.fillRect(0, 0, width, height)

  // Create vertical black gradient
  const blackGradient = ctx.createLinearGradient(0, 0, 0, height)
  blackGradient.addColorStop(0, 'rgba(0,0,0,0)')
  blackGradient.addColorStop(1, 'rgba(0,0,0,1)')
  
  ctx.fillStyle = blackGradient
  ctx.fillRect(0, 0, width, height)
}

// Draw the hue bar
function drawHueBar() {
  const canvas = hueRef.value
  if (!canvas) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  const width = canvas.width
  const height = canvas.height
  const gradient = ctx.createLinearGradient(0, 0, width, 0)
  
  for (let i = 0; i <= 360; i += 60) {
    gradient.addColorStop(i / 360, `hsl(${i}, 100%, 50%)`)
  }
  
  ctx.fillStyle = gradient
  ctx.fillRect(0, 0, width, height)
}

// Handle saturation/value picker click
function handleSaturationClick(e: MouseEvent) {
  const canvas = canvasRef.value
  if (!canvas) return
  
  const rect = canvas.getBoundingClientRect()
  const x = Math.max(0, Math.min(canvas.width, e.clientX - rect.left))
  const y = Math.max(0, Math.min(canvas.height, e.clientY - rect.top))
  
  const s = (x / canvas.width) * 100
  const v = (1 - y / canvas.height) * 100
  
  currentHsv.value = { ...currentHsv.value, s, v }
  emit('update:modelValue', hsvToHex(currentHsv.value.h, s, v))
}

// Handle hue bar click
function handleHueClick(e: MouseEvent) {
  const canvas = hueRef.value
  if (!canvas) return
  
  const rect = canvas.getBoundingClientRect()
  const x = Math.max(0, Math.min(canvas.width, e.clientX - rect.left))
  const h = (x / canvas.width) * 360
  
  currentHsv.value = { ...currentHsv.value, h }
  emit('update:modelValue', hsvToHex(h, currentHsv.value.s, currentHsv.value.v))
  drawSaturationGradient()
}

// Drag handling
const isDraggingSat = ref(false)
const isDraggingHue = ref(false)

function onMouseMove(e: MouseEvent) {
  if (isDraggingSat.value) handleSaturationClick(e)
  if (isDraggingHue.value) handleHueClick(e)
}

function onMouseUp() {
  isDraggingSat.value = false
  isDraggingHue.value = false
}

// Preset colors
const presetColors = [
  '#FFFFFF', '#000000', '#FF0000', '#00FF00', '#0000FF', '#FFFF00',
  '#FF00FF', '#00FFFF', '#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4',
  '#FFEAA7', '#DDA0DD', '#98D8C8', '#F7DC6F', '#BB8FCE', '#85C1E9',
]

function selectPreset(color: string) {
  currentHsv.value = hexToHsv(color)
  emit('update:modelValue', color)
  drawSaturationGradient()
}

function handleHexInput(e: Event) {
  const input = e.target as HTMLInputElement
  let value = input.value.replace(/[^0-9A-Fa-f]/g, '').slice(0, 6)
  if (value.length === 6) {
    const hex = `#${value}`
    currentHsv.value = hexToHsv(hex)
    emit('update:modelValue', hex)
    drawSaturationGradient()
  }
}

// Click outside to close
function handleClickOutside(e: MouseEvent) {
  if (pickerRef.value && !pickerRef.value.contains(e.target as Node) && popupRef.value && !popupRef.value.contains(e.target as Node)) {
    isOpen.value = false
  }
}

// Popup position
const popupRef = ref<HTMLDivElement>()
const triggerRef = ref<HTMLButtonElement>()
const popupPosition = ref({ top: 0, left: 0, openAbove: false })

const POPUP_HEIGHT = 260 
const POPUP_WIDTH = 208 

function updatePopupPosition() {
  if (!triggerRef.value) return
  const rect = triggerRef.value.getBoundingClientRect()
  const viewportHeight = window.innerHeight
  const viewportWidth = window.innerWidth
  
  const spaceBelow = viewportHeight - rect.bottom - 8
  const openAbove = spaceBelow < POPUP_HEIGHT && rect.top > POPUP_HEIGHT
  
  let left = rect.left
  if (left + POPUP_WIDTH > viewportWidth - 8) {
    left = viewportWidth - POPUP_WIDTH - 8
  }
  
  popupPosition.value = {
    top: openAbove ? rect.top - POPUP_HEIGHT - 8 : rect.bottom + 8,
    left,
    openAbove
  }
}

const satCursorX = computed(() => (currentHsv.value.s / 100) * 100)
const satCursorY = computed(() => (1 - currentHsv.value.v / 100) * 100)
const hueCursorX = computed(() => (currentHsv.value.h / 360) * 100)

watch(isOpen, (open) => {
  if (open) {
    updatePopupPosition()
    setTimeout(() => {
      drawSaturationGradient()
      drawHueBar()
    }, 10)
  }
})

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
})

onBeforeUnmount(() => {
  isOpen.value = false
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
})
</script>

<template>
  <div ref="pickerRef" class="relative">
    <button
      ref="triggerRef"
      class="w-4 h-4 rounded cursor-pointer border border-border/50 transition-all duration-200 hover:scale-110 hover:border-border"
      :style="{ backgroundColor: modelValue }"
      @click.stop="isOpen = !isOpen"
    />

    <Teleport to="body">
      <Transition
        enter-active-class="transition-all duration-200 ease-out"
        :enter-from-class="popupPosition.openAbove ? 'opacity-0 scale-95 translate-y-1' : 'opacity-0 scale-95 -translate-y-1'"
        enter-to-class="opacity-100 scale-100 translate-y-0"
        leave-active-class="transition-all duration-150 ease-in"
        leave-from-class="opacity-100 scale-100 translate-y-0"
        :leave-to-class="popupPosition.openAbove ? 'opacity-0 scale-95 translate-y-1' : 'opacity-0 scale-95 -translate-y-1'"
      >
        <div
          v-if="isOpen"
          ref="popupRef"
          class="fixed z-9999 bg-background border border-border rounded shadow-lg p-3 space-y-3 w-52"
          :style="{ top: `${popupPosition.top}px`, left: `${popupPosition.left}px` }"
          @click.stop
        >
          <div class="relative">
            <canvas
              ref="canvasRef"
              width="184"
              height="120"
              class="w-full h-30 cursor-crosshair rounded"
              @mousedown="(e) => { isDraggingSat = true; handleSaturationClick(e) }"
            />
            <!-- Cursor -->
            <div
              class="absolute w-3 h-3 border-2 border-white rounded-full shadow-md pointer-events-none -translate-x-1/2 -translate-y-1/2"
              :style="{ left: `${satCursorX}%`, top: `${satCursorY}%` }"
            />
          </div>

          <!-- Hue bar -->
          <div class="relative">
            <canvas
              ref="hueRef"
              width="184"
              height="12"
              class="w-full h-3 cursor-pointer rounded"
              @mousedown="(e) => { isDraggingHue = true; handleHueClick(e) }"
            />
            <!-- Cursor -->
            <div
              class="absolute top-0 w-1 h-3 bg-white border border-black/20 rounded pointer-events-none -translate-x-1/2"
              :style="{ left: `${hueCursorX}%` }"
            />
          </div>

          <!-- Hex input -->
          <div class="flex items-center gap-2">
            <span class="text-[10px] text-muted-foreground">#</span>
            <input
              type="text"
              :value="displayHex"
              maxlength="6"
              class="flex-1 h-6 px-1.5 text-[10px] bg-muted/30 outline-none font-mono text-foreground uppercase rounded"
              @input="handleHexInput"
            >
            <div
              class="w-6 h-6 rounded border border-border/50"
              :style="{ backgroundColor: modelValue }"
            />
          </div>

          <!-- Preset colors -->
          <div class="grid grid-cols-9 gap-1">
            <button
              v-for="color in presetColors"
              :key="color"
              class="w-4 h-4 rounded border border-border/30 transition-all duration-150 hover:scale-110 hover:border-border"
              :style="{ backgroundColor: color }"
              @click="selectPreset(color)"
            />
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>
