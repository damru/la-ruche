import { ref } from 'vue'

export interface ConfirmOptions {
  title: string
  message: string
  /** Label of the confirm button (default: t('common.confirm')) */
  confirmLabel?: string
  /** 'danger' renders the confirm button in red (default) — use 'primary' for neutral confirmations */
  variant?: 'danger' | 'primary'
}

interface ConfirmState extends ConfirmOptions {
  resolve: (confirmed: boolean) => void
}

const state = ref<ConfirmState | null>(null)

/** Call to open the dialog and await the user's choice. */
function confirm(options: ConfirmOptions): Promise<boolean> {
  return new Promise((resolve) => {
    state.value = { ...options, resolve }
  })
}

function respond(confirmed: boolean) {
  state.value?.resolve(confirmed)
  state.value = null
}

export function useConfirm() {
  return { confirm, respond, state }
}
