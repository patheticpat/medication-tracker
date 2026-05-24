import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUI = defineStore('ui', () => {
  const sortOrder = ref<'alphabetical' | 'urgency'>('urgency')
  const toggleSortOrder = () => {
    sortOrder.value = sortOrder.value === 'alphabetical' ? 'urgency' : 'alphabetical'
  }

  return { sortOrder, toggleSortOrder }
})
