import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'

import './assets/main.css'

import App from './App.vue'
import router from './router'

if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/sw.js')
}

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(PiniaColada)
app.use(router)

app.mount('#app')
