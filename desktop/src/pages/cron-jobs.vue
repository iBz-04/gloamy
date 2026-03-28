<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { computed, onMounted, ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

interface CronJob {
  id: string
  name: string | null
  command: string
  schedule?: {
    kind: 'cron' | 'at' | 'every'
    expr?: string
    tz?: string | null
    at?: string
    every_ms?: number
  }
  expression?: string
  next_run: string
  last_run: string | null
  last_status: string | null
  enabled: boolean
  job_type?: string
  delete_after_run?: boolean
}

interface CronAddBody {
  name?: string
  schedule: string
  command: string
  tz?: string
}

type ScheduleMode = 'daily' | 'weekdays' | 'weekly' | 'monthly' | 'interval' | 'custom'

const auth = useAuthStore()
const jobs = ref<CronJob[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const currentDate = ref(new Date())
const showAddModal = ref(false)
const submitting = ref(false)
const formName = ref('')
const formCommand = ref('')
const formError = ref<string | null>(null)
const selectedJob = ref<CronJob | null>(null)
const deletingJob = ref(false)
const formScheduleMode = ref<ScheduleMode>('daily')
const formTime = ref('09:00')
const formWeekday = ref('1')
const formMonthDay = ref('1')
const formIntervalMinutes = ref('15')
const formCustomCron = ref('')
const localTimeZone = detectLocalTimeZone()
const formTimezone = ref(localTimeZone)

const weekDays = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday']
const weekdayOptions = [
  { value: '0', label: 'Sun', summary: 'Sunday' },
  { value: '1', label: 'Mon', summary: 'Monday' },
  { value: '2', label: 'Tue', summary: 'Tuesday' },
  { value: '3', label: 'Wed', summary: 'Wednesday' },
  { value: '4', label: 'Thu', summary: 'Thursday' },
  { value: '5', label: 'Fri', summary: 'Friday' },
  { value: '6', label: 'Sat', summary: 'Saturday' },
] as const
const scheduleModeOptions = [
  { value: 'daily', label: 'Daily', description: 'Every day at a set time' },
  { value: 'weekdays', label: 'Weekdays', description: 'Monday through Friday' },
  { value: 'weekly', label: 'Weekly', description: 'One day each week' },
  { value: 'monthly', label: 'Monthly', description: 'Same day each month' },
  { value: 'interval', label: 'Interval', description: 'Every few minutes or hours' },
  { value: 'custom', label: 'Custom', description: 'Enter raw cron yourself' },
] as const satisfies ReadonlyArray<{ value: ScheduleMode, label: string, description: string }>
const intervalOptions = [
  { value: '5', label: '5 min' },
  { value: '15', label: '15 min' },
  { value: '30', label: '30 min' },
  { value: '60', label: '1 hour' },
  { value: '120', label: '2 hours' },
] as const
const timezoneSuggestionId = 'cron-timezone-suggestions'
const availableTimeZones = computed(() => {
  const common = [
    localTimeZone,
    'UTC',
    'Europe/London',
    'Europe/Istanbul',
    'America/New_York',
    'America/Los_Angeles',
    'Asia/Tokyo',
    'Asia/Singapore',
    'Australia/Sydney',
  ]
  const supportedValuesOf = (Intl as unknown as {
    supportedValuesOf?: (input: string) => string[]
  }).supportedValuesOf
  const supported = supportedValuesOf ? supportedValuesOf('timeZone') : []

  return Array.from(new Set([...common.filter(Boolean), ...supported]))
})

const currentMonth = computed(() => {
  return currentDate.value.toLocaleDateString('en-US', { month: 'long', year: 'numeric' })
})

const calendarDays = computed(() => {
  const year = currentDate.value.getFullYear()
  const month = currentDate.value.getMonth()

  const firstDay = new Date(year, month, 1)
  const lastDay = new Date(year, month + 1, 0)

  const days: { date: Date, isCurrentMonth: boolean, jobs: CronJob[] }[] = []

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
        d.date.getFullYear() === nextRunDate.getFullYear()
        && d.date.getMonth() === nextRunDate.getMonth()
        && d.date.getDate() === nextRunDate.getDate(),
      )
      if (dayIndex !== -1 && days[dayIndex]) {
        days[dayIndex].jobs.push(job)
      }
    }
  }

  return days
})

function isToday(date: Date) {
  const today = new Date()
  return date.getDate() === today.getDate()
    && date.getMonth() === today.getMonth()
    && date.getFullYear() === today.getFullYear()
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

function detectLocalTimeZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC'
}

function parseTimeParts(value: string): { hour: number, minute: number } | null {
  const [hourRaw, minuteRaw] = value.split(':')
  const hour = Number.parseInt(hourRaw ?? '', 10)
  const minute = Number.parseInt(minuteRaw ?? '', 10)

  if (
    !Number.isInteger(hour)
    || !Number.isInteger(minute)
    || hour < 0
    || hour > 23
    || minute < 0
    || minute > 59
  ) {
    return null
  }

  return { hour, minute }
}

function cronForInterval(totalMinutes: number): string {
  if (!Number.isInteger(totalMinutes) || totalMinutes <= 0)
    return ''

  if (totalMinutes < 60)
    return `*/${totalMinutes} * * * *`

  if (totalMinutes % 60 === 0) {
    const hours = totalMinutes / 60
    return hours === 1 ? '0 * * * *' : `0 */${hours} * * *`
  }

  return ''
}

function scheduleModeButtonClass(mode: ScheduleMode): string {
  if (formScheduleMode.value === mode) {
    return 'border-primary/30 bg-primary/6 text-foreground shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]'
  }

  return 'border-border/45 bg-background/75 text-muted-foreground hover:border-border/70 hover:bg-card/35 hover:text-foreground'
}

function chipButtonClass(active: boolean): string {
  if (active) {
    return 'border-primary/30 bg-primary/6 text-foreground shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]'
  }

  return 'border-border/45 bg-background/75 text-muted-foreground hover:border-border/70 hover:bg-card/35 hover:text-foreground'
}

const normalizedTimezone = computed(() => formTimezone.value.trim())
const selectedWeekday = computed(() => {
  return weekdayOptions.find(option => option.value === formWeekday.value) ?? weekdayOptions[1]
})
const scheduleExpression = computed(() => {
  if (formScheduleMode.value === 'custom')
    return formCustomCron.value.trim()

  const parsedTime = parseTimeParts(formTime.value)
  if (!parsedTime)
    return ''

  const { hour, minute } = parsedTime

  if (formScheduleMode.value === 'daily')
    return `${minute} ${hour} * * *`

  if (formScheduleMode.value === 'weekdays')
    return `${minute} ${hour} * * 1-5`

  if (formScheduleMode.value === 'weekly')
    return `${minute} ${hour} * * ${formWeekday.value}`

  if (formScheduleMode.value === 'monthly') {
    const day = Number.parseInt(formMonthDay.value, 10)
    if (!Number.isInteger(day) || day < 1 || day > 28)
      return ''

    return `${minute} ${hour} ${day} * *`
  }

  if (formScheduleMode.value === 'interval') {
    const interval = Number.parseInt(formIntervalMinutes.value, 10)
    return cronForInterval(interval)
  }

  return ''
})
const scheduleDescription = computed(() => {
  const timezone = normalizedTimezone.value || localTimeZone

  if (formScheduleMode.value === 'custom') {
    return scheduleExpression.value
      ? `Custom cron expression in ${timezone}`
      : 'Enter a custom cron expression'
  }

  if (formScheduleMode.value === 'interval') {
    const match = intervalOptions.find(option => option.value === formIntervalMinutes.value)
    return match
      ? `Runs every ${match.label.toLowerCase()} in ${timezone}`
      : `Runs on an interval in ${timezone}`
  }

  if (formScheduleMode.value === 'monthly')
    return `Runs monthly on day ${formMonthDay.value || '—'} at ${formTime.value || '—'} in ${timezone}`

  if (formScheduleMode.value === 'weekly')
    return `Runs every ${selectedWeekday.value.summary} at ${formTime.value || '—'} in ${timezone}`

  if (formScheduleMode.value === 'weekdays')
    return `Runs Monday through Friday at ${formTime.value || '—'} in ${timezone}`

  return `Runs every day at ${formTime.value || '—'} in ${timezone}`
})

function formatSchedule(job: CronJob): string {
  if (!job.schedule) {
    return job.expression ?? '—'
  }

  if (job.schedule.kind === 'cron') {
    const expr = job.schedule.expr ?? job.expression ?? '—'
    return job.schedule.tz ? `${expr} (${job.schedule.tz})` : expr
  }

  if (job.schedule.kind === 'at') {
    return `Once at ${formatDateTime(job.schedule.at ?? null)}`
  }

  if (job.schedule.kind === 'every') {
    const everyMs = job.schedule.every_ms
    if (!everyMs)
      return 'Interval'

    if (everyMs % 3_600_000 === 0)
      return `Every ${everyMs / 3_600_000}h`
    if (everyMs % 60_000 === 0)
      return `Every ${everyMs / 60_000}m`
    if (everyMs % 1_000 === 0)
      return `Every ${everyMs / 1_000}s`
    return `Every ${everyMs}ms`
  }

  return '—'
}

function resetAddForm() {
  formName.value = ''
  formCommand.value = ''
  formScheduleMode.value = 'daily'
  formTime.value = '09:00'
  formWeekday.value = '1'
  formMonthDay.value = '1'
  formIntervalMinutes.value = '15'
  formCustomCron.value = ''
  formTimezone.value = localTimeZone
  formError.value = null
}

async function addJob() {
  const schedule = scheduleExpression.value.trim()
  if (!schedule || !formCommand.value.trim()) {
    formError.value = 'Choose a schedule and command.'
    return
  }

  if (!normalizedTimezone.value) {
    formError.value = 'Timezone is required.'
    return
  }

  submitting.value = true
  formError.value = null

  const payload: CronAddBody = {
    schedule,
    command: formCommand.value.trim(),
    tz: normalizedTimezone.value,
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
  }
  catch (err: any) {
    formError.value = err?.message || 'Failed to add cron job'
  }
  finally {
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
  }
  catch (err: any) {
    error.value = err?.message || 'Failed to delete cron job'
  }
  finally {
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
    }
    else if (response && 'jobs' in response) {
      jobs.value = response.jobs
    }
    else {
      jobs.value = []
    }
  }
  catch (err: any) {
    console.error('[cron-jobs] fetchJobs error:', err)
    error.value = err.message || 'Failed to load cron jobs'
    jobs.value = []
  }
  finally {
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
            class="p-1.5 rounded-xl hover:bg-card/50 text-muted-foreground hover:text-foreground transition-colors"
            @click="prevMonth"
          >
            <Icon icon="ph:caret-left" class="size-4" />
          </button>
          <h1 class="font-sans text-[18px] font-medium text-foreground min-w-[180px] text-center">
            {{ currentMonth }}
          </h1>
          <button
            class="p-1.5 rounded-xl hover:bg-card/50 text-muted-foreground hover:text-foreground transition-colors"
            @click="nextMonth"
          >
            <Icon icon="ph:caret-right" class="size-4" />
          </button>
          <button
            class="ml-2 px-3 py-1.5 text-[12px] font-medium rounded-xl border border-border/50 hover:bg-card/50 text-muted-foreground hover:text-foreground transition-colors"
            @click="goToToday"
          >
            Today
          </button>
        </div>

        <div class="flex items-center gap-2">
          <!-- Add button -->
          <button
            class="flex items-center gap-1.5 px-3 py-1.5 bg-primary text-primary-foreground rounded-xl text-[12px] font-medium hover:opacity-90 transition-opacity"
            @click="showAddModal = true"
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
        <button class="text-[12px] font-medium hover:underline" @click="fetchJobs">
          Retry
        </button>
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

    <div v-if="showAddModal" class="fixed inset-0 z-40 bg-black/30 backdrop-blur-[1.5px] flex items-center justify-center p-4">
      <div class="w-full max-w-[580px] max-h-[82vh] overflow-y-auto rounded-[20px] border border-border/35 bg-card/92 p-3.5 shadow-[0_20px_55px_-28px_rgba(0,0,0,0.72)] motion-safe:animate-in motion-safe:fade-in-0 motion-safe:zoom-in-95 motion-safe:duration-200 motion-safe:ease-out">
        <div class="flex items-center justify-between mb-2.5">
          <h2 class="font-sans text-[15px] font-medium text-foreground">
            Add Cron Job
          </h2>
          <button
            class="p-1 rounded-lg text-muted-foreground transition-colors duration-150 hover:text-foreground hover:bg-muted/45"
            @click="showAddModal = false; resetAddForm()"
          >
            <Icon icon="ph:x" class="size-4" />
          </button>
        </div>

        <div class="space-y-3.5">
          <label class="block">
            <span class="mb-0.5 block text-[12px] text-muted-foreground">Name (optional)</span>
            <input
              v-model="formName"
              type="text"
              class="w-full rounded-lg border border-border/45 bg-background/90 px-3 py-1.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
              placeholder="Nightly Backup"
            >
          </label>

          <div class="space-y-2">
            <div>
              <p class="text-[13px] font-medium text-foreground">
                Schedule
              </p>
              <p class="text-[11px] text-muted-foreground">
                Pick a pattern and we generate the cron string.
              </p>
            </div>

            <div class="grid grid-cols-2 gap-2 md:grid-cols-3">
              <button
                v-for="option in scheduleModeOptions"
                :key="option.value"
                type="button"
                class="rounded-lg border px-2 py-1.5 text-left transition-all duration-150"
                :class="scheduleModeButtonClass(option.value)"
                @click="formScheduleMode = option.value; formError = null"
              >
                <div class="text-[12px] font-medium">
                  {{ option.label }}
                </div>
                <div class="mt-0.5 text-[9.5px] text-muted-foreground">
                  {{ option.description }}
                </div>
              </button>
            </div>
          </div>

          <div class="space-y-3 rounded-lg border border-border/35 bg-background/70 p-2.5">
            <div v-if="formScheduleMode !== 'interval' && formScheduleMode !== 'custom'" class="grid gap-2.5 sm:grid-cols-2">
              <label class="block">
                <span class="mb-0.5 block text-[12px] text-muted-foreground">Time</span>
                <input
                  v-model="formTime"
                  type="time"
                  class="w-full rounded-lg border border-border/45 bg-background/90 px-3 py-1.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
                >
              </label>

              <label class="block">
                <span class="mb-0.5 block text-[12px] text-muted-foreground">Timezone</span>
                <div class="flex items-center gap-2">
                  <input
                    v-model="formTimezone"
                    :list="timezoneSuggestionId"
                    type="text"
                    class="w-full rounded-lg border border-border/45 bg-background/90 px-3 py-1.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
                    placeholder="Europe/Istanbul"
                  >
                  <button
                    type="button"
                    class="shrink-0 rounded-lg border border-border/45 bg-muted/35 px-2.5 py-1.5 text-[11px] font-medium text-muted-foreground transition-all duration-150 hover:border-border/70 hover:bg-card/45 hover:text-foreground"
                    @click="formTimezone = localTimeZone"
                  >
                    Local
                  </button>
                </div>
                <p class="mt-1 text-[11px] text-muted-foreground">Use an IANA timezone like <code>Europe/Istanbul</code> or <code>America/New_York</code>.</p>
              </label>
            </div>

            <div v-if="formScheduleMode === 'weekly'" class="space-y-1.5">
              <p class="text-[12px] text-muted-foreground">
                Day of week
              </p>
              <div class="grid grid-cols-4 gap-2 sm:grid-cols-7">
                <button
                  v-for="option in weekdayOptions"
                  :key="option.value"
                  type="button"
                  class="rounded-lg border px-2 py-1.5 text-[12px] font-medium transition-all duration-150"
                  :class="chipButtonClass(formWeekday === option.value)"
                  @click="formWeekday = option.value"
                >
                  {{ option.label }}
                </button>
              </div>
            </div>

            <div v-if="formScheduleMode === 'monthly'" class="grid gap-2.5 sm:grid-cols-[minmax(0,160px)_1fr]">
              <label class="block">
                <span class="mb-0.5 block text-[12px] text-muted-foreground">Day of month</span>
                <input
                  v-model="formMonthDay"
                  type="number"
                  min="1"
                  max="28"
                  class="w-full rounded-lg border border-border/45 bg-background/90 px-3 py-1.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
                >
              </label>
              <div class="flex items-end">
                <p class="text-[12px] text-muted-foreground">
                  Limited to 1-28 so it always exists in every month.
                </p>
              </div>
            </div>

            <div v-if="formScheduleMode === 'interval'" class="space-y-3">
              <div class="space-y-1.5">
                <p class="text-[12px] text-muted-foreground">
                  Repeat every
                </p>
                <div class="grid grid-cols-3 gap-2 sm:grid-cols-5">
                  <button
                    v-for="option in intervalOptions"
                    :key="option.value"
                    type="button"
                    class="rounded-lg border px-2.5 py-1.5 text-[12px] font-medium transition-all duration-150"
                    :class="chipButtonClass(formIntervalMinutes === option.value)"
                    @click="formIntervalMinutes = option.value"
                  >
                    {{ option.label }}
                  </button>
                </div>
              </div>

              <label class="block">
                <span class="mb-0.5 block text-[12px] text-muted-foreground">Timezone</span>
                <div class="flex items-center gap-2">
                  <input
                    v-model="formTimezone"
                    :list="timezoneSuggestionId"
                    type="text"
                    class="w-full rounded-lg border border-border/45 bg-background/90 px-3 py-1.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
                    placeholder="Europe/Istanbul"
                  >
                  <button
                    type="button"
                    class="shrink-0 rounded-lg border border-border/45 bg-muted/35 px-2.5 py-1.5 text-[11px] font-medium text-muted-foreground transition-all duration-150 hover:border-border/70 hover:bg-card/45 hover:text-foreground"
                    @click="formTimezone = localTimeZone"
                  >
                    Local
                  </button>
                </div>
                <p class="mt-1 text-[11px] text-muted-foreground">Intervals still use a timezone for consistent next-run display.</p>
              </label>
            </div>

            <div v-if="formScheduleMode === 'custom'" class="space-y-2">
              <label class="block">
                <span class="mb-0.5 block text-[12px] text-muted-foreground">Custom cron expression</span>
                <input
                  v-model="formCustomCron"
                  type="text"
                  class="w-full rounded-lg border border-border/45 bg-background/90 px-3 py-1.5 font-mono text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
                  placeholder="0 9 * * *"
                >
                <p class="mt-1 text-[11px] text-muted-foreground">Format: <code>minute hour day month weekday</code>.</p>
              </label>

              <label class="block">
                <span class="mb-0.5 block text-[12px] text-muted-foreground">Timezone</span>
                <div class="flex items-center gap-2">
                  <input
                    v-model="formTimezone"
                    :list="timezoneSuggestionId"
                    type="text"
                    class="w-full rounded-lg border border-border/45 bg-background/90 px-3 py-1.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
                    placeholder="Europe/Istanbul"
                  >
                  <button
                    type="button"
                    class="shrink-0 rounded-lg border border-border/45 bg-muted/35 px-2.5 py-1.5 text-[11px] font-medium text-muted-foreground transition-all duration-150 hover:border-border/70 hover:bg-card/45 hover:text-foreground"
                    @click="formTimezone = localTimeZone"
                  >
                    Local
                  </button>
                </div>
              </label>
            </div>

            <datalist :id="timezoneSuggestionId">
              <option
                v-for="timezone in availableTimeZones"
                :key="timezone"
                :value="timezone"
              />
            </datalist>

            <div class="rounded-lg border border-primary/15 bg-primary/5 px-2.5 py-2">
              <p class="text-[11px] font-medium text-muted-foreground">
                Schedule preview
              </p>
              <p class="mt-0.5 text-[12.5px] text-foreground">
                {{ scheduleDescription }}
              </p>
              <div class="mt-1.5 rounded-lg border border-border/35 bg-background/80 px-2.5 py-1.5">
                <div class="mb-0.5 text-[10px] text-muted-foreground">
                  Cron expression
                </div>
                <code class="font-mono text-[12px] text-foreground">{{ scheduleExpression || 'Complete the fields to generate a cron expression.' }}</code>
              </div>
            </div>
          </div>

          <label class="block">
            <span class="mb-0.5 block text-[12px] text-muted-foreground">Command</span>
            <input
              v-model="formCommand"
              type="text"
              class="w-full rounded-lg border border-border/45 bg-background/90 px-3 py-1.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
              placeholder="echo hello"
            >
            <p class="mt-1 text-[11px] text-muted-foreground">This is the shell command the daemon will run on that schedule.</p>
          </label>

          <p v-if="formError" class="text-[12px] text-destructive">
            {{ formError }}
          </p>
        </div>

        <div class="mt-3.5 flex items-center justify-end gap-2">
          <button
            class="rounded-lg border border-border/45 px-3 py-1.5 text-[12px] font-medium text-muted-foreground transition-all duration-150 hover:text-foreground hover:bg-muted/45"
            @click="showAddModal = false; resetAddForm()"
          >
            Cancel
          </button>
          <button
            :disabled="submitting"
            class="rounded-lg bg-primary px-3 py-1.5 text-[12px] font-medium text-primary-foreground transition-all duration-150 hover:brightness-105 disabled:opacity-60"
            @click="addJob"
          >
            {{ submitting ? 'Adding...' : 'Add Job' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="selectedJob" class="fixed inset-0 z-40 bg-black/30 flex items-center justify-center p-4">
      <div class="w-full max-w-md rounded-2xl border border-border/40 bg-card p-5 shadow-md">
        <div class="flex items-center justify-between mb-4">
          <h2 class="font-sans text-[16px] font-medium text-foreground">
            Cron Job
          </h2>
          <button
            class="p-1 rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/50"
            @click="selectedJob = null"
          >
            <Icon icon="ph:x" class="size-4" />
          </button>
        </div>

        <div class="space-y-2 text-[13px]">
          <p><span class="text-muted-foreground">Name:</span> {{ jobLabel(selectedJob) }}</p>
          <p><span class="text-muted-foreground">Type:</span> {{ selectedJob.job_type ?? 'shell' }}</p>
          <p><span class="text-muted-foreground">Schedule:</span> {{ formatSchedule(selectedJob) }}</p>
          <p><span class="text-muted-foreground">Command:</span> {{ selectedJob.command }}</p>
          <p><span class="text-muted-foreground">Next run:</span> {{ formatDateTime(selectedJob.next_run) }}</p>
          <p><span class="text-muted-foreground">Last run:</span> {{ formatDateTime(selectedJob.last_run) }}</p>
          <p><span class="text-muted-foreground">Status:</span> {{ selectedJob.last_status ?? 'pending' }}</p>
          <p><span class="text-muted-foreground">Auto-delete:</span> {{ selectedJob.delete_after_run ? 'yes' : 'no' }}</p>
        </div>

        <div class="mt-5 flex items-center justify-between">
          <button
            :disabled="deletingJob"
            class="px-3 py-1.5 rounded-xl bg-destructive/10 text-destructive text-[12px] font-medium border border-destructive/20 disabled:opacity-60"
            @click="deleteSelectedJob"
          >
            {{ deletingJob ? 'Deleting...' : 'Delete Job' }}
          </button>
          <button
            class="px-3 py-1.5 rounded-xl border border-border/50 text-[12px] font-medium text-muted-foreground hover:text-foreground hover:bg-muted/50"
            @click="selectedJob = null"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
