import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'
import { createAuth0 } from '@auth0/auth0-vue'
import './assets/main.css'

import App from './App.vue'
import router from './router'

const auth0 = createAuth0({
  domain: import.meta.env.VITE_AUTH0_DOMAIN,
  clientId: import.meta.env.VITE_AUTH0_CLIENT_ID,
  authorizationParams: {
    audience: import.meta.env.VITE_AUTH0_AUDIENCE,
    redirect_uri: import.meta.env.VITE_AUTH0_CALLBACK_URL,
  },
})

if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/sw.js')
}

const app = createApp(App)

app.use(createPinia()).use(PiniaColada).use(router).use(auth0).mount('#app')
