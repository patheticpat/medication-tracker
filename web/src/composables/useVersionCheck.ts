import { onMounted, onUnmounted } from 'vue'
import { useToast } from './useToast'
import { useI18n } from 'vue-i18n'

declare const __GIT_SHA__: string

export function useVersionCheck() {
  const { addToast } = useToast()
  const { t } = useI18n()

  let timerId: number | undefined = undefined

  const checkVersion = async () => {
    try {
      const r = await fetch(`${window.origin}/version.json`)
      if (r.ok) {
        const version: { sha: string } = await r.json()
        if (version.sha !== __GIT_SHA__) {
          addToast(t('strings.newVersion'), 'info', 0, {
            label: t('strings.reload'),
            onClick: () => window.location.reload(),
          })
        }
      }
    } catch {}
  }

  onMounted(() => {
    checkVersion()
    timerId = setInterval(checkVersion, 5 * 60 * 1000)
  })

  onUnmounted(() => {
    if (timerId !== undefined) {
      clearInterval(timerId)
      timerId = undefined
    }
  })
}
