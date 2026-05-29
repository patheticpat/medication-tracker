self.addEventListener('push', (event) => {
  const data = event.data?.json() ?? {}
  const title = data.title ?? 'Medication Tracker'
  const body = data.body ?? 'Ein Medikament läuft bald ab.'

  event.waitUntil(
    self.registration.showNotification(title, {
      body,
    }),
  )
})

self.addEventListener('notificationclick', (event) => {
  event.notification.close()
  event.waitUntil(clients.openWindow('/'))
})
