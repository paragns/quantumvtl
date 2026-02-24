import { onMounted, onUnmounted } from 'vue'

/**
 * Composable that connects to the WS endpoint and calls `fetchData`
 * on every "refresh" message (plus once on mount). Auto-reconnects.
 * Guards against concurrent fetchData calls and cleans up properly.
 */
export function useWebSocket(fetchData: () => Promise<void>) {
  let ws: WebSocket | null = null
  let timer: number | null = null
  let destroyed = false
  let fetching = false

  function guardedFetch() {
    if (fetching) return
    fetching = true
    fetchData().finally(() => { fetching = false })
  }

  function connect() {
    if (destroyed) return
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
    ws = new WebSocket(`${proto}//${location.host}/api/ws`)
    ws.onmessage = () => { guardedFetch() }
    ws.onclose = () => {
      ws = null
      if (!destroyed) {
        timer = window.setTimeout(() => { connect() }, 5000)
      }
    }
  }

  onMounted(() => {
    guardedFetch()
    connect()
  })

  onUnmounted(() => {
    destroyed = true
    if (timer) clearTimeout(timer)
    ws?.close()
  })
}
