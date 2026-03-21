import { createMemoryHistory, createRouter } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'home',
    component: () => import('@/pages/home.vue'),
  },
  {
    path: '/canvas',
    name: 'canvas',
    component: () => import('@/pages/canvas.vue'),
  },

  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/pages/settings.vue'),
  },
]

const router = createRouter({
  history: createMemoryHistory(),
  routes,
})

export default router
