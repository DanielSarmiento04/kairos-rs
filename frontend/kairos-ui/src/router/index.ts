import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: () => import('../views/Dashboard.vue'),
    },
    {
      path: '/routes',
      name: 'routes',
      component: () => import('../views/Routes.vue'),
    },
    {
      path: '/config',
      name: 'config',
      component: () => import('../views/Config.vue'),
    },
  ],
})

export default router
