<script setup lang="ts">
import { ref, watch } from 'vue'
import { apiFetch } from '../api'
import { useWebSocket } from '../composables/useWebSocket'
import type { DriveDetail } from '../types'

const props = defineProps<{ id: string }>()

const drive = ref<DriveDetail | null>(null)
const error = ref('')

async function fetchData() {
  try {
    const resp = await apiFetch(`/api/vtl/drives/${props.id}`)
    if (resp.ok) {
      drive.value = await resp.json()
      error.value = ''
    } else if (resp.status === 404) {
      error.value = `Drive ${props.id} not found`
    } else {
      error.value = `API error: ${resp.status}`
    }
  } catch {
    error.value = 'Failed to fetch drive data'
  }
}

watch(() => props.id, () => fetchData())
useWebSocket(fetchData)

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i]
}

function formatRate(bytesPerSec: number | null): string {
  if (bytesPerSec == null) return '-'
  return formatBytes(bytesPerSec) + '/s'
}
</script>

<template>
  <div class="drive-detail">
    <router-link to="/ssc" class="back-link">&larr; All Drives</router-link>

    <p v-if="error" class="error">{{ error }}</p>

    <template v-if="drive">
      <!-- Header -->
      <section class="card header-card">
        <div class="header-row">
          <div>
            <h3>Drive {{ drive.id }} &mdash; {{ drive.serial }}</h3>
            <p class="meta">{{ drive.generation }}</p>
          </div>
          <div class="badges">
            <span class="badge" :class="drive.drive_state.toLowerCase()">{{ drive.drive_state }}</span>
            <span v-if="drive.write_protected" class="badge wp">WP</span>
            <span v-if="drive.worm" class="badge worm">WORM</span>
          </div>
        </div>
      </section>

      <!-- Media Info -->
      <section class="card">
        <h3>Media</h3>
        <div v-if="drive.loaded" class="stats">
          <div class="stat"><router-link v-if="drive.barcode" :to="`/media/${drive.barcode}`" class="stat-value barcode-link">{{ drive.barcode }}</router-link><span v-else class="stat-value">N/A</span><span class="stat-label">Barcode</span></div>
          <div class="stat"><span class="stat-value">{{ drive.partition }}</span><span class="stat-label">Partition</span></div>
          <div class="stat"><span class="stat-value">{{ drive.block_number }}</span><span class="stat-label">Block</span></div>
          <div class="stat"><span class="stat-value">{{ drive.file_number }}</span><span class="stat-label">Filemark</span></div>
          <div class="stat" v-if="drive.current_wrap != null"><span class="stat-value">{{ drive.current_wrap }} / {{ drive.total_wraps }}</span><span class="stat-label">Wrap</span></div>
          <div class="stat" v-if="drive.at_bop"><span class="stat-value flag">BOP</span><span class="stat-label">Position</span></div>
          <div class="stat" v-if="drive.at_eod"><span class="stat-value flag">EOD</span><span class="stat-label">Position</span></div>
        </div>
        <p v-else class="empty-state">No media loaded</p>
      </section>

      <!-- Physical Position -->
      <section class="card" v-if="drive.current_wrap != null">
        <h3>Physical Position</h3>
        <div class="stats">
          <div class="stat" v-if="drive.tape_speed != null"><span class="stat-value">{{ drive.tape_speed }}</span><span class="stat-label">Speed</span></div>
        </div>
        <div class="progress-wrap" v-if="drive.position_in_wrap_pct != null">
          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: drive.position_in_wrap_pct + '%' }"></div>
          </div>
          <span class="progress-label">{{ drive.position_in_wrap_pct.toFixed(1) }}% through wrap {{ drive.current_wrap }} of {{ drive.total_wraps }}</span>
        </div>
      </section>

      <!-- Buffer -->
      <section class="card">
        <h3>Buffer</h3>
        <div class="stats">
          <div class="stat"><span class="stat-value">{{ Math.max(drive.write_buffer_pct, drive.read_cache_pct).toFixed(0) }}%</span><span class="stat-label">Buffer</span></div>
          <div class="stat"><span class="stat-value">{{ drive.objects_in_buffer }}</span><span class="stat-label">Objects</span></div>
          <div class="stat"><span class="stat-value">{{ drive.buffer_state }}</span><span class="stat-label">State</span></div>
        </div>
      </section>

      <!-- Performance -->
      <section class="card" v-if="drive.loaded">
        <h3>Performance</h3>
        <div class="stats">
          <div class="stat"><span class="stat-value">{{ formatRate(drive.instantaneous_rate_bytes_sec) }}</span><span class="stat-label">Data Rate</span></div>
          <div class="stat"><span class="stat-value">{{ drive.compression_ratio != null ? drive.compression_ratio.toFixed(2) + ':1' : '-' }}</span><span class="stat-label">Compression</span></div>
          <div class="stat"><span class="stat-value">{{ drive.backhitch_count_this_mount }}</span><span class="stat-label">Backhitches</span></div>
        </div>
      </section>

      <!-- Capacity -->
      <section class="card" v-if="drive.loaded">
        <h3>Capacity</h3>
        <div class="stats">
          <div class="stat"><span class="stat-value">{{ formatBytes(drive.native_bytes_written) }}</span><span class="stat-label">Native Written</span></div>
          <div class="stat"><span class="stat-value">{{ formatBytes(drive.compressed_bytes_written) }}</span><span class="stat-label">Compressed Written</span></div>
          <div class="stat"><span class="stat-value">{{ drive.approximate_remaining_mb != null ? drive.approximate_remaining_mb + ' MB' : '-' }}</span><span class="stat-label">Remaining</span></div>
        </div>
        <div class="progress-wrap" v-if="drive.capacity_used_pct != null">
          <div class="progress-bar">
            <div class="progress-fill capacity" :style="{ width: drive.capacity_used_pct + '%' }"></div>
          </div>
          <span class="progress-label">{{ drive.capacity_used_pct.toFixed(1) }}% used</span>
        </div>
      </section>

      <!-- Lifetime -->
      <section class="card">
        <h3>Lifetime</h3>
        <div class="stats">
          <div class="stat"><span class="stat-value">{{ drive.total_loads }}</span><span class="stat-label">Total Loads</span></div>
          <div class="stat"><span class="stat-value">{{ drive.motion_hours.toFixed(1) }}h</span><span class="stat-label">Motion Hours</span></div>
        </div>
      </section>
    </template>
  </div>
</template>

<style scoped>
.drive-detail { max-width: 900px; }
.back-link { display: inline-block; margin-bottom: 1rem; color: #1a1a2e; text-decoration: none; font-size: 0.9rem; }
.back-link:hover { text-decoration: underline; }
.error { color: #c0392b; margin-bottom: 1rem; }
.empty-state { color: #888; font-style: italic; }

.card { background: #fff; border-radius: 8px; padding: 1rem 1.25rem; margin-bottom: 1rem; box-shadow: 0 1px 4px rgba(0,0,0,0.06); }
.card h3 { margin-bottom: 0.75rem; font-size: 1rem; color: #1a1a2e; }

.header-card .header-row { display: flex; justify-content: space-between; align-items: flex-start; }
.header-card h3 { font-size: 1.1rem; margin-bottom: 0.15rem; }
.meta { color: #888; font-size: 0.85rem; }
.badges { display: flex; gap: 0.4rem; }
.badge { display: inline-block; padding: 0.2rem 0.7rem; border-radius: 999px; font-size: 0.8rem; font-weight: 600; text-transform: capitalize; }
.badge.idle { background: #d4edda; color: #155724; }
.badge.empty { background: #e2e3e5; color: #383d41; }
.badge.reading, .badge.writing { background: #cce5ff; color: #004085; }
.badge.locating, .badge.rewinding, .badge.loading, .badge.unloading { background: #fff3cd; color: #856404; }
.badge.error { background: #f8d7da; color: #721c24; }
.badge.calibrating { background: #d1ecf1; color: #0c5460; }
.badge.wp { background: #f8d7da; color: #721c24; }
.badge.worm { background: #d1ecf1; color: #0c5460; }

.stats { display: flex; gap: 2rem; flex-wrap: wrap; }
.stat { display: flex; flex-direction: column; }
.stat-value { font-size: 1.15rem; font-weight: 700; color: #1a1a2e; }
.stat-value.flag { color: #e67e22; }
.barcode-link { text-decoration: none; font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace; }
.barcode-link:hover { text-decoration: underline; color: #2980b9; }
.stat-label { font-size: 0.78rem; color: #888; }

.progress-wrap { margin-top: 0.75rem; }
.progress-bar { height: 8px; background: #e0e0e0; border-radius: 4px; overflow: hidden; }
.progress-fill { height: 100%; background: #1a1a2e; border-radius: 4px; transition: width 0.3s; }
.progress-fill.capacity { background: #27ae60; }
.progress-label { font-size: 0.78rem; color: #888; margin-top: 0.25rem; display: block; }
</style>
