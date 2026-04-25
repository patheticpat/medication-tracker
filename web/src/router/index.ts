import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/', name: 'dashboard', component: () => import('@/views/DashboardView.vue') },
    { path: '/login', name: 'login', component: () => import('@/views/LoginView.vue') },
    { path: '/register', name: 'register', component: () => import('@/views/RegisterView.vue') },
    { path: '/settings', name: 'settings', component: () => import('@/views/SettingsView.vue') },
    {
      path: '/medications/:id',
      name: 'medications-details',
      component: () => import('@/views/MedicationDetailView.vue'),
    },
  ],
})

router.beforeEach((to, from, next) => {
  const publicRoutes = ['login', 'register']
  const token = localStorage.getItem('token')

  if (!publicRoutes.includes(to.name as string) && !token) {
    next({ name: 'login' })
  } else if (publicRoutes.includes(to.name as string) && token) {
    next({ name: 'dashboard' })
  } else {
    next()
  }
})

export default router
