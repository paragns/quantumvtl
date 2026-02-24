<script setup lang="ts">
import { ref } from 'vue'
import { apiFetch, fetchScsiLog, type ScsiLogSummary } from '../api'
import { useWebSocket } from '../composables/useWebSocket'
import ScsiLogLine from '../components/ScsiLogLine.vue'
import type { DriveSummary } from '../types'

const drives = ref<DriveSummary[]>([])
const driveLogs = ref<Map<number, ScsiLogSummary[]>>(new Map())
const error = ref('')

async function fetchData() {
  try {
    const resp = await apiFetch('/api/vtl/drives')
    if (resp.ok) {
      drives.value = await resp.json()
      error.value = ''
    } else {
      error.value = `API error: ${resp.status}`
    }
  } catch {
    error.value = 'Failed to fetch drive data'
  }
  for (const d of drives.value) {
    const dl = await fetchScsiLog('drive', d.id, 4)
    if (dl) driveLogs.value.set(d.id, dl.entries)
  }
}

useWebSocket(fetchData)
</script>

<template>
  <div class="ssc-view">
    <p v-if="error" class="error">{{ error }}</p>

    <div class="drive-grid">
      <router-link
        v-for="d in drives"
        :key="d.id"
        :to="`/ssc/drive/${d.id}`"
        class="drive-card"
        :class="{ loaded: d.status === 'loaded' }"
      >
        <div class="drive-header">
          <span class="drive-id">Drive {{ d.id }}</span>
          <span class="drive-dot" :class="d.status"></span>
        </div>
        <div class="drive-serial">{{ d.serial }}</div>
        <div v-if="d.status === 'loaded'" class="drive-media">
          <span class="drive-barcode">{{ d.barcode }}</span>
          <span class="drive-pos">pos {{ d.position }} / {{ d.record_count }} rec</span>
        </div>
        <div v-else class="drive-empty">Empty</div>
      </router-link>
    </div>

    <!-- Per-Drive SCSI Activity -->
    <section class="card" v-if="drives.length > 0">
      <h3>SCSI Activity</h3>
      <div v-for="d in drives" :key="d.id" class="drive-log-section">
        <h4><router-link :to="`/device/drive/${d.id}`" class="card-title-link">Drive {{ d.id }}</router-link></h4>
        <div v-if="(driveLogs.get(d.id) ?? []).length > 0">
          <ScsiLogLine
            v-for="e in driveLogs.get(d.id)"
            :key="e.seq"
            :entry="e"
            device-type="drive"
            :device-id="d.id"
          />
        </div>
        <p v-else class="no-activity">No recent activity</p>
      </div>
    </section>

    <p v-if="!drives.length && !error" class="empty-state">No drives configured</p>
  </div>
</template>

<style scoped>
.ssc-view { max-width: 1200px; }
.error { color: #c0392b; margin-bottom: 1rem; }

.drive-grid { display: flex; gap: 0.75rem; flex-wrap: wrap; }
.drive-card {
  border: 2px solid #ddd; border-radius: 8px; padding: 0.8rem 1rem;
  min-width: 150px; flex: 1; max-width: 220px;
  text-decoration: none; color: inherit;
  background: #fff; box-shadow: 0 1px 4px rgba(0,0,0,0.06);
  transition: border-color 0.15s, box-shadow 0.15s;
  cursor: pointer;
}
.drive-card:hover { border-color: #1a1a2e; box-shadow: 0 2px 8px rgba(0,0,0,0.12); }
.drive-card.loaded { border-color: #27ae60; background: #f0faf4; }

.drive-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.3rem; }
.drive-id { font-weight: 600; font-size: 0.9rem; }
.drive-dot { width: 8px; height: 8px; border-radius: 50%; display: inline-block; }
.drive-dot.loaded { background: #27ae60; }
.drive-dot.empty { background: #bbb; }
.drive-serial { font-size: 0.78rem; color: #888; margin-bottom: 0.3rem; }
.drive-media { font-size: 0.85rem; }
.drive-barcode { font-weight: 600; display: block; color: #1a1a2e; }
.drive-pos { font-size: 0.75rem; color: #888; }
.drive-empty { font-size: 0.85rem; color: #bbb; font-style: italic; }

.empty-state { color: #888; text-align: center; margin-top: 3rem; }
.card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; margin-top: 1rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.card h3 { margin-bottom: 0.75rem; font-size: 1rem; color: #1a1a2e; }
.card-title-link { color: #1a1a2e; text-decoration: none; }
.card-title-link:hover { text-decoration: underline; }
.no-activity { color: #bbb; font-style: italic; font-size: 0.85rem; }
.drive-log-section { margin-bottom: 0.75rem; }
.drive-log-section:last-child { margin-bottom: 0; }
.drive-log-section h4 { font-size: 0.85rem; margin-bottom: 0.3rem; }

@media (max-width: 800px) {
  .drive-card { max-width: none; }
}
</style>
