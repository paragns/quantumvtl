<script setup lang="ts">
import { ref } from 'vue'
import { apiFetch } from '../api'
import { useWebSocket } from '../composables/useWebSocket'
import type { SessionInfo } from '../types'

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
</script>

<template>
  <div class="initiators-view">
    <p v-if="error" class="error">{{ error }}</p>

    <section class="card" v-if="sessions.length">
      <h3>Active iSCSI Sessions ({{ sessions.length }})</h3>
      <div class="table-wrap">
        <table>
          <thead>
            <tr>
              <th>Initiator Name</th>
              <th>TSIH</th>
              <th>Remote Address</th>
              <th>Connected Since</th>
              <th>Active Commands</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="s in sessions" :key="s.tsih">
              <td class="mono">{{ s.initiator_name }}</td>
              <td>{{ s.tsih }}</td>
              <td class="mono">{{ s.peer_addr }}</td>
              <td>{{ formatTime(s.connected_since) }}</td>
              <td>{{ s.active_commands }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <div v-else-if="!error" class="empty-state">
      <p>No active iSCSI sessions</p>
    </div>
  </div>
</template>

<style scoped>
.initiators-view { max-width: 1200px; }
.error { color: #c0392b; margin-bottom: 1rem; }

.card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; margin-bottom: 1rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.card h3 { margin-bottom: 0.75rem; font-size: 1rem; color: #1a1a2e; }

.table-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: 0.85rem; }
th { text-align: left; padding: 0.4rem 0.6rem; border-bottom: 2px solid #e0e0e0; color: #888; font-weight: 600; font-size: 0.78rem; text-transform: uppercase; }
td { padding: 0.4rem 0.6rem; border-bottom: 1px solid #f0f0f0; }
.mono { font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace; font-size: 0.82rem; }

.empty-state { text-align: center; color: #888; margin-top: 3rem; font-size: 0.95rem; }
</style>
