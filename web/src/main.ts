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
    home: {
      description:
        'Behalte deine Medikamente im Blick. Lass dich benachrichtigen bevor sie dir ausgehen und verpasse keine Nachbestellung.',
      goToDashboard: 'Zum Dashboard',
      getStarted: 'Loslegen',
    },
    strings: {
      app: 'Medication Tracker',
      reload: 'Neu laden',
      newVersion: 'Neue Version verfügbar',
      day: 'Tag | Tage',
      daysOfWeek: 'Wochentage',
      unknown: 'unbekannt',
      error: 'Etwas ist schiefgelaufen. Bitte versuche es später noch einmal.',
      unknownError: 'Unbekannter Fehler',
      save: 'Speichern',
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
      saved: 'Einstellungen gespeichert',
      saveFailed: 'Einstellungen konnten nicht gespeichert werden',
      time: 'Uhrzeit',
      versions: 'Versionen',
      notificationsEnabled: 'Benachrichtigungen aktiviert',
      notificationsEnableFailed: 'Benachrichtigungen konnten nicht aktiviert werden',
      notificationsDisabled: 'Benachrichtigungen deaktiviert',
      notificationsDisableFailed: 'Benachrichtigungen konnten nicht deaktiviert werden',
      testNotificationSent: 'Test-Benachrichtigung gesendet',
      testNotificationFailed: 'Test-Benachrichtigung konnte nicht gesendet werden',
    },
    dashboard: {
      noMedications: 'Noch keine Medikamente. Hinzufügen mit dem + Button.',
      remaining: 'verbleibend',
      runningLow: 'Niedriger Bestand',
      medicationsSnoozed: 'Ein Medikament stummgeschaltet | {n} Medikamente stummgeschaltet',
      sort: {
        alphabetical: 'A-Z',
        urgency: 'Vorrat',
      },
      noMedicationsThisWeek: 'Keine Bestellung fällig',
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
      snooze: 'Stumm',
      unsnooze: 'Erinnern',
      snoozeAll: 'Alle stumm',
      stock: 'Aktueller Bestand',
      daysRemaining: 'Verbleibende Tage',
      note: 'Notiz (optional)',
      refill: 'Nachfüllen',
      recount: 'Nachzählen',
      setBaseline: 'Bestand korrigieren',
      history: 'Verlauf',
      refillLogged: 'Nachfüllung erfasst',
      stockUpdated: 'Bestand aktualisiert',
      deleted: '"{name}" gelöscht',
      deleteFailed: '"{name}" konnte nicht gelöscht werden',
      added: '"{name}" hinzugefügt',
      addFailed: '"{name}" konnte nicht hinzugefügt werden',
      updateFailed: '"{name}" konnte nicht aktualisiert werden',
      updated: '"{name}" aktualisiert',
      snoozeFailed: 'Stummschaltung konnte nicht geändert werden',
      action: {
        refill: 'Eingang',
        baseline: 'Bestand',
      },
    },
  },
}
const i18n = createI18n({
  locale: 'de',
  messages,
  numberFormats: {
    de: {
      decimal: {
        style: 'decimal',
        minimumFractionDigits: 0,
        maximumFractionDigits: 1,
      },
      dosage: {
        style: 'decimal',
        minimumFractionDigits: 0,
        maximumFractionDigits: 6,
      },
    },
  },
})

if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('/sw.js')
}

const app = createApp(App)

app.use(createPinia()).use(PiniaColada).use(router).use(auth0).use(i18n).mount('#app')
