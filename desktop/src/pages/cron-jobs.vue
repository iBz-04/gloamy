<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAuthStore } from '@/stores/auth'

interface CronJob {
  id: string
  name: string | null
  command: string
  next_run: string
  last_run: string | null
  last_status: string | null
  enabled: boolean
}

interface CronAddBody {
  name?: string
  schedule: string
  command: string
}

const auth = useAuthStore()
const jobs = ref<CronJob[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const currentDate = ref(new Date())
const showAddModal = ref(false)
const submitting = ref(false)
const formName = ref('')
const formSchedule = ref('')
const formCommand = ref('')
const formError = ref<string | null>(null)
const selectedJob = ref<CronJob | null>(null)
const deletingJob = ref(false)

const weekDays = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']

const currentMonth = computed(() => {
  return currentDate.value.toLocaleDateString('en-US', { month: 'long', year: 'numeric' })
})

const calendarDays = computed(() => {
  const year = currentDate.value.getFullYear()
  const month = currentDate.value.getMonth()
  
  const firstDay = new Date(year, month, 1)
  const lastDay = new Date(year, month + 1, 0)
  
  const days: { date: Date; isCurrentMonth: boolean; jobs: CronJob[] }[] = []
  
  // Add days from previous month to fill the first week
  const startPadding = firstDay.getDay()
  for (let i = startPadding - 1; i >= 0; i--) {
    const date = new Date(year, month, -i)
    days.push({ date, isCurrentMonth: false, jobs: [] })
  }
  
  // Add days of current month
  for (let d = 1; d <= lastDay.getDate(); d++) {
    const date = new Date(year, month, d)
    days.push({ date, isCurrentMonth: true, jobs: [] })
  }
  
  // Add days from next month to complete the grid (6 rows)
  const endPadding = 42 - days.length
  for (let i = 1; i <= endPadding; i++) {
    const date = new Date(year, month + 1, i)
    days.push({ date, isCurrentMonth: false, jobs: [] })
  }
  
  // Assign jobs to their scheduled days
  for (const job of jobs.value) {
    if (job.next_run) {
      const nextRunDate = new Date(job.next_run)
      if (Number.isNaN(nextRunDate.getTime())) {
        continue
      }
      const dayIndex = days.findIndex(d => 
        d.date.getFullYear() === nextRunDate.getFullYear() &&
        d.date.getMonth() === nextRunDate.getMonth() &&
        d.date.getDate() === nextRunDate.getDate()
      )
      if (dayIndex !== -1 && days[dayIndex]) {
        days[dayIndex].jobs.push(job)
      }
    }
  }
  
  return days
})

const isToday = (date: Date) => {
  const today = new Date()
  return date.getDate() === today.getDate() &&
    date.getMonth() === today.getMonth() &&
    date.getFullYear() === today.getFullYear()
}

function prevMonth() {
  const d = new Date(currentDate.value)
  d.setMonth(d.getMonth() - 1)
  currentDate.value = d
}

function nextMonth() {
  const d = new Date(currentDate.value)
  d.setMonth(d.getMonth() + 1)
  currentDate.value = d
}

function goToToday() {
  currentDate.value = new Date()
}

function jobPillClass(job: CronJob): string {
  if (!job.enabled)
    return 'bg-muted text-muted-foreground border-border'

  const status = (job.last_status ?? '').toLowerCase()
  if (status === 'ok' || status === 'success')
    return 'bg-emerald-500/10 text-emerald-600 border-emerald-500/20'
  if (status === 'error' || status === 'failed')
    return 'bg-destructive/10 text-destructive border-destructive/20'

  return 'bg-primary/10 text-primary border-primary/20'
}

function jobLabel(job: CronJob): string {
  const name = (job.name ?? '').trim()
  if (name.length > 0)
    return name
  return job.command
}

function formatDateTime(value: string | null): string {
  if (!value)
    return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime()))
    return '—'
  return date.toLocaleString()
}

function resetAddForm() {
  formName.value = ''
  formSchedule.value = ''
  formCommand.value = ''
  formError.value = null
}

async function addJob() {
  if (!formSchedule.value.trim() || !formCommand.value.trim()) {
    formError.value = 'Schedule and command are required.'
    return
  }

  submitting.value = true
  formError.value = null

  const payload: CronAddBody = {
    schedule: formSchedule.value.trim(),
    command: formCommand.value.trim(),
  }

  if (formName.value.trim()) {
    payload.name = formName.value.trim()
  }

  try {
    await auth.fetchWithAuth('/api/cron', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    })
    showAddModal.value = false
    resetAddForm()
    await fetchJobs()
  } catch (err: any) {
    formError.value = err?.message || 'Failed to add cron job'
  } finally {
    submitting.value = false
  }
}

async function deleteSelectedJob() {
  if (!selectedJob.value)
    return

  deletingJob.value = true
  try {
    await auth.fetchWithAuth(`/api/cron/${encodeURIComponent(selectedJob.value.id)}`, {
      method: 'DELETE',
    })
    selectedJob.value = null
    await fetchJobs()
  } catch (err: any) {
    error.value = err?.message || 'Failed to delete cron job'
  } finally {
    deletingJob.value = false
  }
}

async function fetchJobs() {
  loading.value = true
  error.value = null
  try {
    const response = await auth.fetchWithAuth<{ jobs: CronJob[] } | CronJob[]>('/api/cron')
    if (Array.isArray(response)) {
      jobs.value = response
    } else if (response && 'jobs' in response) {
      jobs.value = response.jobs
    } else {
      jobs.value = []
    }
  } catch (err: any) {
    console.error('[cron-jobs] fetchJobs error:', err)
    error.value = err.message || 'Failed to load cron jobs'
    jobs.value = []
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchJobs()
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <!-- Header -->
    <div class="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border/30">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-3">
          <button
            @click="prevMonth"
            class="p-1.5 rounded-xl hover:bg-card/50 text-muted-foreground hover:text-foreground transition-colors"
          >
            <Icon icon="ph:caret-left" class="size-4" />
          </button>
          <h1 class="font-sans text-[18px] font-medium text-foreground min-w-[180px] text-center">
            {{ currentMonth }}
          </h1>
          <button
            @click="nextMonth"
            class="p-1.5 rounded-xl hover:bg-card/50 text-muted-foreground hover:text-foreground transition-colors"
          >
            <Icon icon="ph:caret-right" class="size-4" />
          </button>
          <button
            @click="goToToday"
            class="ml-2 px-3 py-1.5 text-[12px] font-medium rounded-xl border border-border/50 hover:bg-card/50 text-muted-foreground hover:text-foreground transition-colors"
          >
            Today
          </button>
        </div>

        <div class="flex items-center gap-2">
          <!-- Add button -->
          <button
            @click="showAddModal = true"
            class="flex items-center gap-1.5 px-3 py-1.5 bg-primary text-primary-foreground rounded-xl text-[12px] font-medium hover:opacity-90 transition-opacity"
          >
            <Icon icon="ph:plus" class="size-3.5" />
            Add Job
          </button>
        </div>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <Icon icon="ph:circle-notch" class="size-6 animate-spin text-muted-foreground" />
    </div>

    <!-- Calendar view -->
    <div v-else class="flex-1 overflow-hidden flex flex-col px-6 pb-4">
      <div v-if="error" class="mb-3 rounded-xl border border-destructive/20 bg-destructive/5 px-3 py-2 text-[12px] text-destructive flex items-center justify-between">
        <span>{{ error }}</span>
        <button @click="fetchJobs" class="text-[12px] font-medium hover:underline">Retry</button>
      </div>
      <div class="flex-1 rounded-2xl border border-border/30 overflow-hidden bg-card/10">
        <!-- Week day headers -->
        <div class="grid grid-cols-7 border-b border-border/30">
          <div
            v-for="day in weekDays"
            :key="day"
            class="px-3 py-2 text-[11px] font-medium text-muted-foreground text-center"
          >
            {{ day.slice(0, 3) }}
          </div>
        </div>

        <!-- Calendar grid -->
        <div class="h-full grid grid-cols-7 grid-rows-6 overflow-hidden">
          <div
            v-for="(day, index) in calendarDays"
            :key="index"
            class="border-b border-r border-border/20 p-1.5 overflow-hidden flex flex-col"
            :class="{
              'bg-card/30': !day.isCurrentMonth,
              'bg-background': day.isCurrentMonth,
            }"
          >
            <!-- Date number -->
            <div class="flex items-center justify-between mb-1">
              <span
                class="text-[11px] font-medium w-6 h-6 flex items-center justify-center rounded-full"
                :class="{
                  'text-muted-foreground/50': !day.isCurrentMonth,
                  'text-foreground': day.isCurrentMonth && !isToday(day.date),
                  'bg-primary text-primary-foreground': isToday(day.date),
                }"
              >
                {{ day.date.getDate() }}
              </span>
            </div>

            <!-- Jobs for this day -->
            <div class="flex-1 space-y-0.5 overflow-y-auto">
              <div
                v-for="job in day.jobs.slice(0, 3)"
                :key="job.id"
                class="px-1.5 py-0.5 text-[10px] font-medium rounded-lg border truncate cursor-pointer hover:opacity-80 transition-opacity"
                :class="jobPillClass(job)"
                :title="jobLabel(job)"
                @click="selectedJob = job"
              >
                {{ jobLabel(job) }}
              </div>
              <div
                v-if="day.jobs.length > 3"
                class="text-[10px] text-muted-foreground px-1.5 cursor-pointer hover:text-foreground"
              >
                +{{ day.jobs.length - 3 }} more
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div v-if="showAddModal" class="fixed inset-0 z-40 bg-black/30 flex items-center justify-center p-4">
      <div class="w-full max-w-md rounded-2xl border border-border/40 bg-card p-5 shadow-md">
        <div class="flex items-center justify-between mb-4">
          <h2 class="font-sans text-[16px] font-medium text-foreground">Add Cron Job</h2>
          <button
            @click="showAddModal = false; resetAddForm()"
            class="p-1 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/50"
          >
            <Icon icon="ph:x" class="size-4" />
          </button>
        </div>

        <div class="space-y-3">
          <label class="block">
            <span class="block text-[12px] text-muted-foreground mb-1">Name (optional)</span>
            <input
              v-model="formName"
              type="text"
              class="w-full rounded-xl border border-border/50 bg-background px-3 py-2 text-[13px]"
              placeholder="Nightly Backup"
            />
          </label>

          <label class="block">
            <span class="block text-[12px] text-muted-foreground mb-1">Cron Schedule</span>
            <input
              v-model="formSchedule"
              type="text"
              class="w-full rounded-xl border border-border/50 bg-background px-3 py-2 text-[13px]"
              placeholder="0 9 * * *"
            />
          </label>

          <label class="block">
            <span class="block text-[12px] text-muted-foreground mb-1">Command</span>
            <input
              v-model="formCommand"
              type="text"
              class="w-full rounded-xl border border-border/50 bg-background px-3 py-2 text-[13px]"
              placeholder="echo hello"
            />
          </label>

          <p v-if="formError" class="text-[12px] text-destructive">{{ formError }}</p>
        </div>

        <div class="mt-5 flex items-center justify-end gap-2">
          <button
            @click="showAddModal = false; resetAddForm()"
            class="px-3 py-1.5 rounded-xl border border-border/50 text-[12px] font-medium text-muted-foreground hover:text-foreground hover:bg-muted/50"
          >
            Cancel
          </button>
          <button
            @click="addJob"
            :disabled="submitting"
            class="px-3 py-1.5 rounded-xl bg-primary text-primary-foreground text-[12px] font-medium disabled:opacity-60"
          >
            {{ submitting ? 'Adding...' : 'Add Job' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="selectedJob" class="fixed inset-0 z-40 bg-black/30 flex items-center justify-center p-4">
      <div class="w-full max-w-md rounded-2xl border border-border/40 bg-card p-5 shadow-md">
        <div class="flex items-center justify-between mb-4">
          <h2 class="font-sans text-[16px] font-medium text-foreground">Cron Job</h2>
          <button
            @click="selectedJob = null"
            class="p-1 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/50"
          >
            <Icon icon="ph:x" class="size-4" />
          </button>
        </div>

        <div class="space-y-2 text-[13px]">
          <p><span class="text-muted-foreground">Name:</span> {{ jobLabel(selectedJob) }}</p>
          <p><span class="text-muted-foreground">Command:</span> {{ selectedJob.command }}</p>
          <p><span class="text-muted-foreground">Next run:</span> {{ formatDateTime(selectedJob.next_run) }}</p>
          <p><span class="text-muted-foreground">Last run:</span> {{ formatDateTime(selectedJob.last_run) }}</p>
          <p><span class="text-muted-foreground">Status:</span> {{ selectedJob.last_status ?? 'pending' }}</p>
        </div>

        <div class="mt-5 flex items-center justify-between">
          <button
            @click="deleteSelectedJob"
            :disabled="deletingJob"
            class="px-3 py-1.5 rounded-xl bg-destructive/10 text-destructive text-[12px] font-medium border border-destructive/20 disabled:opacity-60"
          >
            {{ deletingJob ? 'Deleting...' : 'Delete Job' }}
          </button>
          <button
            @click="selectedJob = null"
            class="px-3 py-1.5 rounded-xl border border-border/50 text-[12px] font-medium text-muted-foreground hover:text-foreground hover:bg-muted/50"
          >
            Close
          </button>
        </div>
      </div>
    </div>

  </div>
</template>

