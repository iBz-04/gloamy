<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '@/stores/auth'
import { useSettingsStore } from '@/stores/settings'
import { useColorMode } from '@vueuse/core'
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu'

const auth = useAuthStore()
const settingsStore = useSettingsStore()
const mode = useColorMode()

const selectedTheme = ref<'light'|'dark'|'auto'>('auto')
const selectedLanguage = ref('English')

const updatesEnabled = ref(true)
const emailEnabled = ref(true)

const languageOptions = [
  { value: 'English', label: 'English' },
  { value: 'zh-CN', label: '简体中文' },
  { value: 'ja', label: '日本語' },
  { value: 'ru', label: 'Русский' },
]

onMounted(async () => {
  const t = await settingsStore.getSetting<string>('theme')
  if (t === 'light' || t === 'dark' || t === 'auto') {
    selectedTheme.value = t
  }
  const l = await settingsStore.getSetting<string>('language')
  if (l) selectedLanguage.value = l
})

function handleThemeChange(theme: 'light' | 'dark' | 'auto') {
  selectedTheme.value = theme
  mode.value = theme
  settingsStore.setSetting<string>('theme', theme)
}

function handleLanguageChange(langValue: string) {
  selectedLanguage.value = langValue
  settingsStore.setSetting<string>('language', langValue)
}

const logout = async () => {
  await auth.logout()
}
</script>

<template>
  <div class="h-full flex flex-col overflow-y-auto bg-background text-[15px]">
    <!-- Header -->
    <div class="px-6 pt-6 pb-4">
      <h1 class="text-[16px] font-semibold text-foreground tracking-tight">Settings</h1>
    </div>

    <!-- Content -->
    <div class="px-6 pb-12 max-w-2xl space-y-10">
      
      <!-- General Section -->
      <section class="space-y-4">
        <h2 class="text-[13px] font-medium text-muted-foreground">General</h2>
        
        <div>
          <label class="block text-[14px] font-medium text-foreground mb-2">Language</label>
          <div class="relative w-48">
            <DropdownMenu>
              <DropdownMenuTrigger as-child>
                <button
                  class="w-full bg-card border border-border/60 hover:border-border/80 text-foreground text-[14px] rounded-lg px-3 py-2 flex items-center justify-between focus:outline-none focus:ring-1 focus:ring-primary/50 transition-colors"
                >
                  {{ languageOptions.find(opt => opt.value === selectedLanguage)?.label || selectedLanguage }}
                  <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-muted-foreground"><path d="m6 9 6 6 6-6"/></svg>
                </button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start" class="w-48">
                <DropdownMenuItem
                  v-for="option in languageOptions"
                  :key="option.value"
                  @click="handleLanguageChange(option.value)"
                  class="flex items-center justify-between"
                >
                  {{ option.label }}
                  <svg v-if="selectedLanguage === option.value" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-primary"><path d="M20 6 9 17l-5-5"/></svg>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
        </div>
      </section>

      <!-- Appearance Section -->
      <section class="space-y-4">
        <h2 class="text-[13px] font-medium text-muted-foreground">Appearance</h2>
        
        <div class="flex flex-wrap gap-5">
          <!-- Light Mode Card -->
          <div class="flex flex-col items-center gap-2">
            <button
              @click="handleThemeChange('light')"
              class="relative w-[136px] h-[92px] rounded-[18px] bg-[#F8F9FA] border-[3px] transition-all p-1.5 focus:outline-none overflow-hidden"
              :class="selectedTheme === 'light' ? 'border-primary' : 'border-[#E2E8F0] hover:border-border'"
            >
              <div class="w-full h-full bg-white rounded-xl shadow-[0_1px_3px_rgba(0,0,0,0.05)] border border-[#E2E8F0] flex p-[5px] gap-1.5 pointer-events-none">
                <div class="w-[30%] h-full bg-[#F1F5F9] rounded border border-[#E2E8F0]"></div>
                <div class="flex-1 h-full bg-[#FFFFFF] rounded border border-[#E2E8F0] flex flex-col gap-1.5 p-1.5">
                  <div class="w-[60%] h-1.5 bg-[#CBD5E1] rounded-full"></div>
                  <div class="w-full h-1 bg-[#F1F5F9] rounded-full mt-1"></div>
                  <div class="w-[85%] h-1 bg-[#F1F5F9] rounded-full"></div>
                </div>
              </div>
            </button>
            <span class="text-[13px] font-medium text-muted-foreground" :class="{ 'text-foreground': selectedTheme === 'light' }">Light</span>
          </div>

          <!-- Dark Mode Card -->
          <div class="flex flex-col items-center gap-2">
            <button
              @click="handleThemeChange('dark')"
              class="relative w-[136px] h-[92px] rounded-[18px] bg-[#27272A] border-[3px] transition-all p-1.5 focus:outline-none overflow-hidden"
              :class="selectedTheme === 'dark' ? 'border-primary' : 'border-[#3F3F46] hover:border-[#52525B]'"
            >
              <div class="w-full h-full bg-[#09090B] rounded-xl shadow-[0_1px_4px_rgba(0,0,0,0.5)] border border-[#3F3F46] flex p-[5px] gap-1.5 pointer-events-none">
                <div class="w-[30%] h-full bg-[#18181B] rounded border border-[#3F3F46]"></div>
                <div class="flex-1 h-full bg-[#09090B] rounded border border-[#3F3F46] flex flex-col gap-1.5 p-1.5">
                  <div class="w-[60%] h-1.5 bg-[#52525B] rounded-full"></div>
                  <div class="w-full h-1 bg-[#27272A] rounded-full mt-1"></div>
                  <div class="w-[85%] h-1 bg-[#27272A] rounded-full"></div>
                </div>
              </div>
            </button>
            <span class="text-[13px] font-medium text-muted-foreground" :class="{ 'text-foreground': selectedTheme === 'dark' }">Dark</span>
          </div>

          <!-- System Mode Card -->
          <div class="flex flex-col items-center gap-2">
            <button
              @click="handleThemeChange('auto')"
              class="relative w-[136px] h-[92px] rounded-[18px] border-[3px] transition-all p-0 focus:outline-none overflow-hidden"
              :class="selectedTheme === 'auto' ? 'border-primary' : 'border-[#E2E8F0] dark:border-[#3F3F46] hover:border-border'"
            >
              <div class="absolute inset-0 flex rounded-xl overflow-hidden m-[3px]">
                <!-- Left (Light) -->
                <div class="w-1/2 h-full bg-[#F8F9FA] pt-1.5 pl-1.5 pb-1.5 pr-[2px]">
                   <div class="w-full h-full bg-white rounded-l-xl opacity-[0.98] shadow-sm border-y border-l border-[#E2E8F0] flex p-[5px] gap-1.5 pointer-events-none">
                     <div class="w-full h-full bg-[#F1F5F9] rounded border border-[#E2E8F0]"></div>
                   </div>
                </div>
                <!-- Right (Dark) -->
                <div class="w-1/2 h-full bg-[#27272A] pt-1.5 pr-1.5 pb-1.5 pl-[2px]">
                   <div class="w-full h-full bg-[#09090B] rounded-r-xl opacity-[0.98] shadow-[0_1px_3px_rgba(0,0,0,0.5)] border-y border-r border-[#3F3F46] flex p-[5px] pointer-events-none justify-end">
                     <div class="w-full h-full bg-[#09090B] rounded border border-[#3F3F46] flex flex-col gap-1.5 p-1.5">
                       <div class="w-[80%] h-1.5 bg-[#52525B] rounded-full"></div>
                       <div class="w-full h-1 bg-[#27272A] rounded-full mt-1"></div>
                     </div>
                   </div>
                </div>
              </div>
            </button>
            <span class="text-[13px] font-medium text-muted-foreground" :class="{ 'text-foreground': selectedTheme === 'auto' }">Follow System</span>
          </div>
        </div>
      </section>

      <!-- Communication preferences Section -->
      <section class="space-y-6 pt-4">
        <h2 class="text-[13px] font-medium text-muted-foreground">Communication preferences</h2>
        
        <div class="space-y-6">
          <div class="flex items-start justify-between">
            <div class="pr-6">
              <p class="text-[14px] font-medium text-foreground">Receive product updates</p>
              <p class="text-[13px] text-muted-foreground mt-1">Receive early access to feature releases and success stories to optimize your workflow.</p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer flex-shrink-0 mt-1">
              <input type="checkbox" v-model="updatesEnabled" class="sr-only peer">
              <div class="w-9 h-5 bg-border rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary"></div>
            </label>
          </div>
          
          <div class="flex items-start justify-between">
            <div class="pr-6">
              <p class="text-[14px] font-medium text-foreground">Email me when my queued task starts</p>
              <p class="text-[13px] text-muted-foreground mt-1">When enabled, we'll send you a timely email once your task finishes queuing and begins processing.</p>
            </div>
            <label class="relative inline-flex items-center cursor-pointer flex-shrink-0 mt-1">
              <input type="checkbox" v-model="emailEnabled" class="sr-only peer">
              <div class="w-9 h-5 bg-border rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-primary"></div>
            </label>
          </div>
        </div>
      </section>
      
      <hr class="border-border/30" />
      
      <!-- Session Management / Account Management -->
      <section>
        <div class="flex items-center justify-between py-1">
          <p class="text-[14px] font-medium text-foreground">Account management</p>
          <div class="flex items-center gap-3">
             <button
               class="px-5 py-1.5 bg-card border border-border/80 hover:bg-muted text-foreground transition-colors rounded-[8px] text-[13px] font-medium"
             >
               Manage
             </button>
             <button
              @click="logout"
              class="px-5 py-1.5 bg-destructive/10 border border-transparent hover:border-destructive/20 text-destructive transition-colors rounded-[8px] text-[13px] font-medium"
            >
              Sign out device
            </button>
          </div>
        </div>
      </section>

    </div>
  </div>
</template>
