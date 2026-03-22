import { createMemoryHistory, createRouter } from 'vue-router'
import PlaceholderPage from '@/pages/placeholder.vue'

const routes = [
  {
    path: '/',
    name: 'dashboard',
    component: () => import('@/pages/home.vue'),
    meta: {
      title: 'Dashboard',
      description: 'Monitor your agent runtime at a glance.',
    },
  },
  {
    path: '/agent-chat',
    name: 'agent-chat',
    component: PlaceholderPage,
    meta: {
      title: 'Agent Chat',
      description: 'Chat directly with your Gloamy agent.',
    },
  },
  {
    path: '/tools',
    name: 'tools',
    component: PlaceholderPage,
    meta: {
      title: 'Tools',
      description: 'Configure and inspect available tools.',
    },
  },
  {
    path: '/cron-jobs',
    name: 'cron-jobs',
    component: PlaceholderPage,
    meta: {
      title: 'Cron Jobs',
      description: 'Manage scheduled automations and routines.',
    },
  },
  {
    path: '/integrations',
    name: 'integrations',
    component: PlaceholderPage,
    meta: {
      title: 'Integrations',
      description: 'Connect channels, providers, and peripherals.',
    },
  },
  {
    path: '/memory',
    name: 'memory',
    component: PlaceholderPage,
    meta: {
      title: 'Memory',
      description: 'Inspect or purge conversation memory stores.',
    },
  },
  {
    path: '/configuration',
    name: 'configuration',
    component: PlaceholderPage,
    meta: {
      title: 'Configuration',
      description: 'Review runtime configuration and overrides.',
    },
  },
  {
    path: '/cost-tracking',
    name: 'cost-tracking',
    component: PlaceholderPage,
    meta: {
      title: 'Cost Tracking',
      description: 'Track token spend and billing per provider.',
    },
  },
  {
    path: '/logs',
    name: 'logs',
    component: PlaceholderPage,
    meta: {
      title: 'Logs',
      description: 'Trace actions, events, and transport logs.',
    },
  },
  {
    path: '/doctor',
    name: 'doctor',
    component: PlaceholderPage,
    meta: {
      title: 'Doctor',
      description: 'Run diagnostics to catch misconfiguration.',
    },
  },
  {
    path: '/authentication',
    name: 'authentication',
    component: PlaceholderPage,
    meta: {
      title: 'Authentication & Pairing',
      description: 'Pair new clients and manage credentials.',
    },
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/pages/settings.vue'),
    meta: {
      title: 'Settings & Theme',
      description: 'Adjust appearance and global preferences.',
    },
  },
]

const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

export default router
