// src/stores/auth.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useQueryCache } from '@pinia/colada'

export const useAuthStore = defineStore('auth', () => {
  const isLoggedIn = ref(localStorage.getItem('token') !== null)

  function login(token: string) {
    localStorage.setItem('token', token)
    isLoggedIn.value = true
  }

  function logout() {
    localStorage.removeItem('token')
    const queryCache = useQueryCache()
    queryCache.invalidateQueries()
    isLoggedIn.value = false
  }

  return { isLoggedIn, login, logout }
})
