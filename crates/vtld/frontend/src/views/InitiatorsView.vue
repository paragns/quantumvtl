<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { apiFetch } from '../api'
import type { ScsiLogSummary } from '../api'
import { useWebSocket } from '../composables/useWebSocket'
import type { SessionInfo } from '../types'

const router = useRouter()
const sessions = ref<SessionInfo[]>([])
const error = ref('')

async function fetchData() {
  try {
    const resp = await apiFetch('/api/vtl/sessions')
    if (resp.ok) {
      sessions.value = await resp.json()
      error.value = ''
    } else {
      error.value = `API error: ${resp.status}`
    }
  } catch {
    error.value = 'Failed to fetch session data'
  }
}

useWebSocket(fetchData)

function formatTime(iso: string): string {
  try {
    return new Date(iso).toLocaleString()
  } catch {
    return iso
  }
}

function formatLogTime(ts: string): string {
  try {
    const d = new Date(ts)
    const hh = String(d.getHours()).padStart(2, '0')
    const mm = String(d.getMinutes()).padStart(2, '0')
    const ss = String(d.getSeconds()).padStart(2, '0')
    const ms = String(d.getMilliseconds()).padStart(3, '0')
    return `${hh}:${mm}:${ss}.${ms}`
  } catch { return ts }
}

function formatDuration(us: number): string {
  if (us < 1000) return `${us}us`
  if (us < 1000000) return `${(us / 1000).toFixed(1)}ms`
  return `${(us / 1000000).toFixed(2)}s`
}

function formatOpcode(op: number): string {
  return `0x${op.toString(16).toUpperCase().padStart(2, '0')}`
}

function formatBytes(n: number): string {
  if (n === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.min(Math.floor(Math.log(n) / Math.log(1024)), units.length - 1)
  const val = n / Math.pow(1024, i)
  return `${i === 0 ? val : val.toFixed(1)} ${units[i]}`
}

function cmdPath(tsih: number, e: ScsiLogSummary): string {
  return `/initiators/${tsih}/cmd/${e.seq}`
}
</script>

<template>
  <div class="initiators-view">
    <p v-if="error" class="error">{{ error }}</p>

    <div v-if="sessions.length" class="sessions">
      <section class="session-card" v-for="s in sessions" :key="s.tsih">
        <div class="session-header">
          <span class="initiator-name">{{ s.initiator_name }}</span>
          <span class="tsih-badge">TSIH {{ s.tsih }}</span>
          <span v-if="s.active_commands > 0" class="active-badge">{{ s.active_commands }} active</span>
        </div>

        <div class="connections">
          <div class="connection-card" v-for="c in s.connections" :key="c.cid">
            <div class="conn-header">
              <span class="peer-addr mono">{{ c.peer_addr }}</span>
              <span class="connected-since">since {{ formatTime(c.connected_since) }}</span>
              <span v-if="c.active_commands > 0" class="active-badge small">{{ c.active_commands }} active</span>
            </div>

            <div class="stats-row">
              <div class="stat">
                <span class="stat-label">RX Cmds</span>
                <span class="stat-value">{{ c.rx_commands.toLocaleString() }}</span>
              </div>
              <div class="stat">
                <span class="stat-label">RX Bytes</span>
                <span class="stat-value">{{ formatBytes(c.rx_bytes) }}</span>
              </div>
              <div class="stat">
                <span class="stat-label">TX Cmds</span>
                <span class="stat-value">{{ c.tx_commands.toLocaleString() }}</span>
              </div>
              <div class="stat">
                <span class="stat-label">TX Bytes</span>
                <span class="stat-value">{{ formatBytes(c.tx_bytes) }}</span>
              </div>
            </div>

            <div v-if="c.scsi_log.length > 0" class="scsi-log-section">
              <div class="scsi-log-label">Recent SCSI Commands</div>
              <table class="scsi-table">
                <thead>
                  <tr>
                    <th>Time</th>
                    <th>Opcode</th>
                    <th>Command</th>
                    <th>Status</th>
                    <th>Duration</th>
                    <th>Out</th>
                    <th>In</th>
                  </tr>
                </thead>
                <tbody>
                  <tr
                    v-for="e in c.scsi_log"
                    :key="e.seq"
                    class="clickable"
                    @click="router.push(cmdPath(s.tsih, e))"
                  >
                    <td class="mono">{{ formatLogTime(e.timestamp) }}</td>
                    <td class="mono">{{ formatOpcode(e.opcode) }}</td>
                    <td class="cmd-name">{{ e.opcode_name }}</td>
                    <td>
                      <span class="status-badge" :class="{ good: e.status === 0, error: e.status !== 0 }">
                        {{ e.status_name }}
                      </span>
                    </td>
                    <td class="mono">{{ formatDuration(e.duration_us) }}</td>
                    <td class="mono">{{ e.data_out_len }}</td>
                    <td class="mono">{{ e.data_in_len }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </section>
    </div>

    <div v-else-if="!error" class="empty-state">
      <p>No active iSCSI sessions</p>
    </div>
  </div>
</template>

<style scoped>
.initiators-view { max-width: 1200px; }
.error { color: #c0392b; margin-bottom: 1rem; }

.sessions { display: flex; flex-direction: column; gap: 1rem; }

.session-card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.session-header { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.75rem; flex-wrap: wrap; }
.initiator-name { font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace; font-size: 0.9rem; font-weight: 600; color: #1a1a2e; }
.tsih-badge { background: #e8eaf6; color: #3949ab; padding: 0.1rem 0.5rem; border-radius: 3px; font-size: 0.75rem; font-weight: 600; }
.active-badge { background: #fff3e0; color: #e65100; padding: 0.1rem 0.5rem; border-radius: 3px; font-size: 0.75rem; font-weight: 600; }
.active-badge.small { font-size: 0.7rem; padding: 0.05rem 0.4rem; }

.connections { display: flex; flex-direction: column; gap: 0.75rem; }

.connection-card { background: #f8f9fa; border-radius: 6px; padding: 0.75rem 1rem; border: 1px solid #e9ecef; }
.conn-header { display: flex; align-items: center; gap: 0.75rem; margin-bottom: 0.5rem; flex-wrap: wrap; }
.peer-addr { font-size: 0.85rem; font-weight: 600; color: #333; }
.connected-since { font-size: 0.78rem; color: #888; }

.stats-row { display: flex; gap: 1.5rem; margin-bottom: 0.5rem; flex-wrap: wrap; }
.stat { display: flex; flex-direction: column; }
.stat-label { font-size: 0.68rem; font-weight: 600; text-transform: uppercase; color: #999; }
.stat-value { font-size: 0.85rem; font-weight: 600; color: #333; font-family: 'SF Mono', 'Consolas', monospace; }

.scsi-log-section { margin-top: 0.5rem; }
.scsi-log-label { font-size: 0.72rem; font-weight: 600; text-transform: uppercase; color: #888; margin-bottom: 0.35rem; }

.scsi-table { width: 100%; border-collapse: collapse; font-size: 0.78rem; }
.scsi-table th { text-align: left; padding: 0.3rem 0.4rem; border-bottom: 1px solid #dee2e6; font-size: 0.7rem; color: #888; text-transform: uppercase; }
.scsi-table td { padding: 0.25rem 0.4rem; border-bottom: 1px solid #e9ecef; }
.scsi-table .clickable { cursor: pointer; }
.scsi-table .clickable:hover { background: #e9ecef; }
.mono { font-family: 'SF Mono', 'Consolas', 'Liberation Mono', monospace; }
.cmd-name { font-weight: 600; color: #1a1a2e; }
.status-badge { display: inline-block; padding: 0.1rem 0.4rem; border-radius: 3px; font-size: 0.72rem; font-weight: 600; }
.status-badge.good { background: #d4edda; color: #155724; }
.status-badge.error { background: #f8d7da; color: #721c24; }

.empty-state { text-align: center; color: #888; margin-top: 3rem; font-size: 0.95rem; }
</style>
