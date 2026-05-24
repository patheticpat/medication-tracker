import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUI = defineStore('ui', () => {
  const sortOrder = ref<'alphabetical' | 'urgency'>('alphabetical')
  const toggleSortOrder = () => {
    sortOrder.value = sortOrder.value === 'alphabetical' ? 'urgency' : 'alphabetical'
  }

  return { sortOrder, toggleSortOrder }
})
