import { onMounted, onUnmounted } from 'vue'

/**
 * Composable that connects to the WS endpoint and calls `fetchData`
 * on every "refresh" message (plus once on mount). Auto-reconnects.
 */
export function useWebSocket(fetchData: () => Promise<void>) {
  let ws: WebSocket | null = null
  let timer: number | null = null

  function connect() {
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
    ws = new WebSocket(`${proto}//${location.host}/api/ws`)
    ws.onmessage = () => { fetchData() }
    ws.onclose = () => {
      ws = null
      timer = window.setTimeout(() => { connect() }, 5000)
    }
  }

  onMounted(() => {
    fetchData()
    connect()
  })

  onUnmounted(() => {
    ws?.close()
    if (timer) clearTimeout(timer)
  })
}
