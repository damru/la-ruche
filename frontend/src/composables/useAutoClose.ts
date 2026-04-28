import { SUCCESS_AUTO_CLEAR_MS } from './useFormFeedback'

/**
 * DDR-006 — Auto-close timer management for modal success windows.
 *
 * `schedule(fn)` — starts the 3 s countdown; `fn` is called when it expires.
 * `forceExpire()` — cancels any pending timer and runs `fn` immediately.
 *                   No-op when no timer is running (safe to call unconditionally).
 *
 * Pair one `useAutoClose()` instance with each modal that shows a success
 * window.  Pass the same callback to `schedule()` that you want to run on
 * both natural expiry and early (manual) close.
 */
export function useAutoClose() {
  let _timer: ReturnType<typeof setTimeout> | undefined
  let _pendingFn: (() => void) | undefined

  function schedule(fn: () => void, delay = SUCCESS_AUTO_CLEAR_MS): void {
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
