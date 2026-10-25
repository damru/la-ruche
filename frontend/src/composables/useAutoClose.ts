import { useUiStore } from '@/stores/ui'

/**
 * DDR-006 — Auto-close timer management for modal success windows.
 *
 * `schedule(fn)` — starts the countdown; `fn` is called when it expires.
 *                  The default delay is the user preference
 *                  (0–10 s, default 3 s); 0 closes immediately without a
 *                  success window.
 * `forceExpire()` — cancels any pending timer and runs `fn` immediately.
 *                   No-op when no timer is running (safe to call unconditionally).
 *
 * Pair one `useAutoClose()` instance with each modal that shows a success
 * window.  Pass the same callback to `schedule()` that you want to run on
 * both natural expiry and early (manual) close.
 */
export function useAutoClose() {
  const ui = useUiStore()

  let _timer: ReturnType<typeof setTimeout> | undefined
  let _pendingFn: (() => void) | undefined

  function schedule(fn: () => void, delay = ui.autoCloseDelayMs): void {
    // 0 = immediate close, no success window.
    if (delay <= 0) {
      fn()
      return
    }
    // In practice, early-close guards in action functions ensure schedule() is
    // never called while a timer is already running, so the previous fn is not lost.
    _pendingFn = fn
    clearTimeout(_timer)
    _timer = setTimeout(() => {
      _timer = undefined
      _pendingFn = undefined
      fn()
    }, delay)
  }

  /** Cancel the pending timer and run the scheduled callback immediately. */
  function forceExpire(): void {
    if (_timer !== undefined) {
      clearTimeout(_timer)
      _timer = undefined
      const fn = _pendingFn
      _pendingFn = undefined
      fn?.()
    }
  }

  function isPending(): boolean {
    return _timer !== undefined
  }

  return { schedule, forceExpire, isPending }
}
