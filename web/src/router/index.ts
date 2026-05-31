import { authGuard } from '@auth0/auth0-vue'
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'home', component: () => import('@/views/HomeView.vue') },
    { path: '/callback', name: 'callback', component: () => import('@/views/CallbackView.vue') },
    {
      path: '/medications',
      name: 'dashboard',
      component: () => import('@/views/DashboardView.vue'),
      beforeEnter: authGuard,
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
      beforeEnter: authGuard,
    },
    {
      path: '/medications/:id',
      name: 'medications-details',
      component: () => import('@/views/MedicationDetailView.vue'),
      beforeEnter: authGuard,
    },
  ],
})

export default router
