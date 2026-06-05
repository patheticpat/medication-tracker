import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { PiniaColada } from '@pinia/colada'
import { createAuth0 } from '@auth0/auth0-vue'
import { createI18n } from 'vue-i18n'
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

const messages = {
  de: {
    navigation: { login: 'Anmelden', logout: 'Abmelden', settings: 'Einstellungen' },
    clipboard: {
      copy: 'Kopieren',
      copied: 'Kopiert!',
    },
    strings: {
      app: 'Medication Tracker',
      day: 'Tag | Tage',
      daysOfWeek: 'Wochentage',
      unknown: 'unbekannt',
      week: 'Woche',
      weekdays: {
        monday: 'Montag',
        tuesday: 'Dienstag',
        wednesday: 'Mittwoch',
        thursday: 'Donnerstag',
        friday: 'Freitag',
        saturday: 'Samstag',
        sunday: 'Sonntag',
        on: {
          monday: 'montags',
          tuesday: 'dienstags',
          wednesday: 'mittwochs',
          thursday: 'donnerstags',
          friday: 'freitags',
          saturday: 'samstags',
          sunday: 'sonntags',
        },
        short: {
          monday: 'Mo',
          tuesday: 'Di',
          wednesday: 'Mi',
          thursday: 'Do',
          friday: 'Fr',
          saturday: 'Sa',
          sunday: 'So',
        },
      },
    },
    settings: {
      notifications: 'Push Benachrichtigungen',
      enableNotifications: 'Benachrichtigungen aktivieren',
      disableNotifications: 'Benachrichtigungen deaktivieren',
      testNotifications: 'Test-Benachrichtigung senden',
      notificationsUnsupported: 'Dieser Browser unterstützt leider keine Push Benachrichtigungen.',
      notificationsBlocked: 'Erlaube Benachrichtigungen in deinem Browser.',
      notificationsExplanation:
        'Erhalte eine Push Notification wenn dein Vorrat einen kritischen Stand unterschreitet.',
      save: 'Einstellungen speichern',
      time: 'Uhrzeit',
      versions: 'Versionen',
    },
    dashboard: {
      noMedications: 'Noch keine Medikamente. Hinzufügen mit dem + Button.',
      remaining: 'verbleibend',
      runningLow: 'Niedriger Bestand',
      sort: {
        alphabetical: 'A-Z',
        urgency: 'Vorrat',
      },
    },
    medication: {
      add: 'Medikament hinzufügen',
      edit: 'Medikament bearbeiten',
      dailyDose: '{dosage} pro Tag',
      weeklyDose: '{dosage} pro Woche ({day})',
      editButton: 'Bearbeiten',
      delete: 'Löschen',
      confirmDelete: 'wirklich?',
      name: 'Bezeichnung',
      unit: 'Einheit',
      unitSingular: 'Einheit (Einzahl)',
      schedule: 'Zeitplan',
      daily: 'täglich',
      weekly: 'wöchentlich',
      threshold: 'Warnschwelle',
      thresholdInDays: 'Warnschwelle (in Tagen)',
      save: 'Änderungen speichern',
      defaultUnitSingular: 'Tablette',
      defaultUnit: 'Tabletten',
      initialStock: 'Anfangsbestand',
      snooze: 'Snooze',
      unsnooze: 'Unsnooze',
      snoozeAll: 'Alle snoozen',
      stock: 'Aktueller Bestand',
      daysRemaining: 'Verbleibende Tage',
      note: 'Notiz (optional)',
      refill: 'Nachfüllen',
      recount: 'Nachzählen',
      setBaseline: 'Bestand korrigieren',
      action: {
        refill: 'Eingang',
        baseline: 'Bestand',
      },
    },
  },
}
const i18n = createI18n({ locale: 'de', messages })

if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/sw.js')
}

const app = createApp(App)

app.use(createPinia()).use(PiniaColada).use(router).use(auth0).use(i18n).mount('#app')
