<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { apiFetch, fetchScsiLog, type ScsiLogSummary } from '../api'
import { useWebSocket } from '../composables/useWebSocket'
import ScsiLogLine from '../components/ScsiLogLine.vue'
import type { ChangerDetail, DriveDetail } from '../types'

const changer = ref<ChangerDetail | null>(null)
const changerLog = ref<ScsiLogSummary[]>([])
const drives = ref<DriveDetail[]>([])
const driveLogs = ref<Record<number, ScsiLogSummary[]>>({})
const error = ref('')

async function fetchData() {
  try {
    const resp = await apiFetch('/api/vtl/changer')
    if (resp.ok) {
      changer.value = await resp.json()
      error.value = ''
    } else {
      error.value = `API error: ${resp.status}`
    }
  } catch {
    error.value = 'Failed to fetch data'
  }

  // Fetch changer SCSI log
  const cl = await fetchScsiLog('changer', 0, 4)
  if (cl) changerLog.value = cl.entries

  // Fetch full detail for each drive
  if (changer.value) {
    const driveCount = changer.value.num_drives
    const driveResults: DriveDetail[] = []
    const logResults: Record<number, ScsiLogSummary[]> = {}

    const fetches = Array.from({ length: driveCount }, (_, i) =>
      Promise.all([
        apiFetch(`/api/vtl/drives/${i}`).then(r => r.ok ? r.json() : null),
        fetchScsiLog('drive', i, 4),
      ]).then(([detail, log]) => {
        if (detail) driveResults.push(detail)
        if (log) logResults[i] = log.entries
      })
    )
    await Promise.all(fetches)
    driveResults.sort((a, b) => a.id - b.id)
    drives.value = driveResults
    driveLogs.value = logResults
  }
}

useWebSocket(fetchData)

const nowMs = ref(Date.now())
let elapsedTimer: ReturnType<typeof setInterval> | null = null
let pendingRefetch = false

onMounted(() => {
  elapsedTimer = setInterval(() => {
    nowMs.value = Date.now()
    // Re-fetch while changer is busy: picks up robot_operation on first tick,
    // detects completion on subsequent ticks (WS notifications may be dropped
    // by guardedFetch if a fetch is already in-flight).
    const state = changer.value?.state?.toLowerCase()
    if (state === 'moving' || state === 'scanning') {
      if (!pendingRefetch) {
        pendingRefetch = true
        fetchData().finally(() => { pendingRefetch = false })
      }
    }
  }, 1000)
})

onUnmounted(() => {
  if (elapsedTimer) clearInterval(elapsedTimer)
})

const robotSnapTransition = ref(false)
let lastRobotStartMs = 0

watch(() => changer.value?.robot_operation?.started_at_ms, (ms) => {
  if (ms != null && ms !== lastRobotStartMs) {
    lastRobotStartMs = ms
    robotSnapTransition.value = true
    requestAnimationFrame(() => { robotSnapTransition.value = false })
  }
})

const robotProgressPct = computed(() => {
  const op = changer.value?.robot_operation
  if (!op) return 0
  const elapsed = (nowMs.value - op.started_at_ms) / 1000
  return Math.min(100, (elapsed / op.estimated_secs) * 100)
})

const robotElapsedSecs = computed(() => {
  const op = changer.value?.robot_operation
  if (!op) return 0
  return Math.max(0, (nowMs.value - op.started_at_ms) / 1000)
})

const robotDescription = computed(() => {
  const op = changer.value?.robot_operation
  if (!op) return ''
  if (op.kind === 'moving' && op.source != null && op.dest != null) {
    const src = '0x' + op.source.toString(16).padStart(4, '0').toUpperCase()
    const dst = '0x' + op.dest.toString(16).padStart(4, '0').toUpperCase()
    return `Moving medium ${src} \u2192 ${dst}`
  }
  if (op.kind === 'scanning') return 'Scanning inventory'
  return 'Robot operation in progress'
})

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
  <div class="devices-view">
    <p v-if="error" class="error">{{ error }}</p>

    <template v-if="changer">
      <!-- ===== CHANGER CARD (full width) ===== -->
      <section class="card changer-card">
        <!-- Identity header -->
        <div class="changer-header">
          <div>
            <h2 class="changer-title">{{ changer.vendor }} {{ changer.product }}</h2>
            <p class="changer-meta">S/N: {{ changer.serial }} &middot; FW: {{ changer.firmware_version }}</p>
          </div>
          <span class="badge" :class="changer.state.toLowerCase()">{{ changer.state }}</span>
        </div>

        <!-- Status Details sub-card -->
        <div class="sub-card">
          <h3>Status</h3>
          <div class="stats">
            <div class="stat">
              <span class="stat-value">{{ changer.num_drives }}</span>
              <span class="stat-label">Drives</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ changer.num_slots }}</span>
              <span class="stat-label">Slots</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ changer.num_import_export }}</span>
              <span class="stat-label">I/E Ports</span>
            </div>
            <div class="stat">
              <span class="stat-value">0x{{ changer.picker_position.toString(16).padStart(4, '0').toUpperCase() }}</span>
              <span class="stat-label">Picker Pos</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ changer.total_moves }}</span>
              <span class="stat-label">Total Moves</span>
            </div>
            <div class="stat">
              <span class="stat-value" :class="{ warn: changer.prevent_medium_removal }">{{ changer.prevent_medium_removal ? 'Locked' : 'Unlocked' }}</span>
              <span class="stat-label">Medium Removal</span>
            </div>
            <div v-if="changer.robot_operation" class="stat robot-stat">
              <div class="robot-inline">
                <span class="robot-inline-desc">{{ robotDescription }}</span>
                <div class="robot-inline-bar">
                  <div class="robot-inline-fill" :class="{ 'no-transition': robotSnapTransition }" :style="{ width: robotProgressPct + '%' }"></div>
                </div>
                <span class="robot-inline-time" v-if="robotProgressPct < 100">{{ robotElapsedSecs.toFixed(1) }}s / {{ changer.robot_operation.estimated_secs.toFixed(1) }}s</span>
                <span class="robot-inline-time" v-else>{{ robotElapsedSecs.toFixed(1) }}s — taking longer than expected</span>
              </div>
            </div>
          </div>
          <div v-if="changer.active_alerts.length" class="alert-row">
            <span v-for="a in changer.active_alerts" :key="a" class="alert-badge">TapeAlert #{{ a }}</span>
          </div>
        </div>

        <!-- Environmental sub-card -->
        <div class="sub-card">
          <h3>Environmental</h3>
          <div class="stats">
            <div class="stat">
              <span class="stat-value">{{ changer.temperature_c }}&deg;C</span>
              <span class="stat-label">Temperature</span>
            </div>
            <div class="stat">
              <span class="stat-value">{{ changer.humidity_pct }}%</span>
              <span class="stat-label">Humidity</span>
            </div>
          </div>
        </div>

        <!-- SCSI Activity sub-card -->
        <div class="sub-card">
          <h3><router-link to="/device/changer" class="card-title-link">SCSI Activity</router-link></h3>
          <div v-if="changerLog.length > 0">
            <ScsiLogLine
              v-for="e in changerLog"
              :key="e.seq"
              :entry="e"
              device-type="changer"
              :device-id="0"
            />
          </div>
          <p v-else class="no-activity">No recent activity</p>
        </div>
      </section>

      <!-- ===== DRIVE CARDS (multi-column grid) ===== -->
      <div class="drive-grid" v-if="drives.length > 0">
        <section v-for="d in drives" :key="d.id" class="card drive-card">
          <!-- Drive header -->
          <div class="drive-header">
            <div>
              <h3 class="drive-title">
                <router-link :to="`/ssc/drive/${d.id}`" class="drive-title-link">Drive {{ d.id }}</router-link>
              </h3>
              <p class="drive-meta">{{ d.serial }} &middot; {{ d.generation }}</p>
            </div>
            <div class="badges">
              <span class="badge" :class="d.drive_state.toLowerCase()">{{ d.drive_state }}</span>
              <span v-if="d.write_protected" class="badge wp">WP</span>
              <span v-if="d.worm" class="badge worm">WORM</span>
            </div>
          </div>

          <!-- Media & Position -->
          <div class="drive-body">
            <div class="drive-stats">
              <div class="stat">
                <router-link v-if="d.barcode" :to="`/media/${d.barcode}`" class="stat-value barcode-link">{{ d.barcode }}</router-link>
                <span v-else class="stat-value dim">--</span>
                <span class="stat-label">Media</span>
              </div>
              <div class="stat">
                <span class="stat-value">{{ d.loaded ? d.block_number : '--' }}</span>
                <span class="stat-label">Block</span>
              </div>
              <div class="stat">
                <span class="stat-value">{{ d.loaded ? d.file_number : '--' }}</span>
                <span class="stat-label">Filemark</span>
              </div>
              <div class="stat" v-if="d.current_wrap != null">
                <span class="stat-value">{{ d.current_wrap }} / {{ d.total_wraps }}</span>
                <span class="stat-label">Wrap</span>
              </div>
              <div class="stat">
                <span class="stat-value" :class="{ flag: d.at_bop || d.at_eod }">{{ d.at_bop ? 'BOP' : d.at_eod ? 'EOD' : '--' }}</span>
                <span class="stat-label">Position</span>
              </div>
            </div>

            <!-- Performance / Buffer summary -->
            <div class="drive-stats">
              <div class="stat">
                <span class="stat-value">{{ d.loaded ? formatRate(d.instantaneous_rate_bytes_sec) : '--' }}</span>
                <span class="stat-label">Data Rate</span>
              </div>
              <div class="stat">
                <span class="stat-value">{{ d.loaded ? Math.max(d.write_buffer_pct, d.read_cache_pct).toFixed(0) + '%' : '--' }}</span>
                <span class="stat-label">Buffer</span>
              </div>
              <div class="stat">
                <span class="stat-value">{{ d.loaded ? d.backhitch_count_this_mount : '--' }}</span>
                <span class="stat-label">Backhitches</span>
              </div>
            </div>

            <!-- Capacity bar -->
            <div class="progress-wrap">
              <div class="progress-bar">
                <div class="progress-fill capacity" :style="{ width: (d.capacity_used_pct ?? 0) + '%' }"></div>
              </div>
              <span class="progress-label" v-if="d.capacity_used_pct != null">{{ d.capacity_used_pct.toFixed(1) }}% used &middot; {{ d.approximate_remaining_mb != null ? d.approximate_remaining_mb + ' MB remaining' : '' }}</span>
              <span class="progress-label" v-else>No media</span>
            </div>
          </div>

          <!-- Per-drive SCSI Activity -->
          <div class="drive-scsi">
            <h4><router-link :to="`/device/drive/${d.id}`" class="card-title-link">SCSI Activity</router-link></h4>
            <div v-if="(driveLogs[d.id] ?? []).length > 0">
              <ScsiLogLine
                v-for="e in driveLogs[d.id]"
                :key="e.seq"
                :entry="e"
                device-type="drive"
                :device-id="d.id"
              />
            </div>
            <p v-else class="no-activity">No recent activity</p>
          </div>
        </section>
      </div>
    </template>
  </div>
</template>

<style scoped>
.devices-view { max-width: 1400px; }
.error { color: #c0392b; margin-bottom: 1rem; }

/* ===== Cards ===== */
.card {
  background: #fff;
  border-radius: 8px;
  padding: 1rem 1.25rem;
  margin-bottom: 1rem;
  box-shadow: 0 1px 4px rgba(0,0,0,0.06);
}
.card h3 { margin-bottom: 0.5rem; font-size: 0.95rem; color: #1a1a2e; }

/* ===== Changer Card ===== */
.changer-card { padding: 1.25rem 1.5rem; }
.changer-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 1rem; }
.changer-title { font-size: 1.2rem; font-weight: 700; color: #1a1a2e; margin-bottom: 0.15rem; }
.changer-meta { color: #888; font-size: 0.85rem; }

/* Sub-cards inside changer */
.sub-card {
  background: #f8f9fa;
  border: 1px solid #e9ecef;
  border-radius: 6px;
  padding: 0.85rem 1rem;
  margin-top: 0.75rem;
}
.sub-card h3 { font-size: 0.85rem; color: #555; text-transform: uppercase; letter-spacing: 0.03em; margin-bottom: 0.5rem; }

/* ===== Stats ===== */
.stats { display: flex; gap: 1.5rem; flex-wrap: wrap; }
.stat { display: flex; flex-direction: column; }
.stat-value { font-size: 1.1rem; font-weight: 700; color: #1a1a2e; }
.stat-value.warn { color: #c0392b; }
.stat-value.flag { color: #e67e22; }
.stat-label { font-size: 0.72rem; color: #888; text-transform: uppercase; letter-spacing: 0.02em; }

/* ===== Badges ===== */
.badge { display: inline-block; padding: 0.2rem 0.7rem; border-radius: 999px; font-size: 0.8rem; font-weight: 600; text-transform: capitalize; }
.badge.ready { background: #d4edda; color: #155724; }
.badge.initializing, .badge.scanning, .badge.moving { background: #fff3cd; color: #856404; }
.badge.notready { background: #f8d7da; color: #721c24; }
.badge.idle { background: #d4edda; color: #155724; }
.badge.empty { background: #e2e3e5; color: #383d41; }
.badge.reading, .badge.writing { background: #cce5ff; color: #004085; }
.badge.locating, .badge.rewinding, .badge.loading, .badge.unloading { background: #fff3cd; color: #856404; }
.badge.error { background: #f8d7da; color: #721c24; }
.badge.calibrating { background: #d1ecf1; color: #0c5460; }
.badge.wp { background: #f8d7da; color: #721c24; }
.badge.worm { background: #d1ecf1; color: #0c5460; }
.badges { display: flex; gap: 0.4rem; flex-wrap: wrap; }

/* ===== Alerts ===== */
.alert-row { margin-top: 0.6rem; display: flex; gap: 0.5rem; flex-wrap: wrap; }
.alert-badge { background: #f8d7da; color: #721c24; padding: 0.2rem 0.6rem; border-radius: 4px; font-size: 0.8rem; font-weight: 600; }

/* ===== Drive Grid ===== */
.drive-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 1rem;
  margin-bottom: 1rem;
}
@media (max-width: 800px) {
  .drive-grid { grid-template-columns: 1fr; }
}

/* ===== Drive Card ===== */
.drive-card { padding: 1rem; margin-bottom: 0; min-height: 280px; display: flex; flex-direction: column; }
.drive-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 0.5rem; }
.drive-title { font-size: 1rem; font-weight: 700; margin-bottom: 0.1rem; }
.drive-title-link { color: #1a1a2e; text-decoration: none; }
.drive-title-link:hover { text-decoration: underline; }
.drive-meta { color: #888; font-size: 0.78rem; }

.drive-body { border-top: 1px solid #f0f0f0; padding-top: 0.6rem; flex: 1; }
.drive-stats { display: flex; gap: 1rem; flex-wrap: wrap; margin-bottom: 0.5rem; }
.drive-stats .stat-value { font-size: 0.95rem; }
.drive-stats .stat-value.dim { color: #ccc; }
.drive-stats .stat-label { font-size: 0.68rem; }

.drive-scsi { border-top: 1px solid #f0f0f0; padding-top: 0.6rem; margin-top: 0.5rem; }
.drive-scsi h4 { font-size: 0.78rem; color: #555; text-transform: uppercase; letter-spacing: 0.03em; margin-bottom: 0.3rem; }

/* ===== Progress bars ===== */
.progress-wrap { margin-top: 0.5rem; }
.progress-bar { height: 6px; background: #e0e0e0; border-radius: 3px; overflow: hidden; }
.progress-fill { height: 100%; background: #1a1a2e; border-radius: 3px; transition: width 0.3s; }
.progress-fill.capacity { background: #27ae60; }
.progress-label { font-size: 0.72rem; color: #888; margin-top: 0.2rem; display: block; }

/* ===== Inline Robot Progress ===== */
.robot-stat { flex: 1 1 100%; }
.robot-inline { display: flex; align-items: center; gap: 0.6rem; }
.robot-inline-desc { font-size: 0.82rem; font-weight: 600; color: #1a1a2e; white-space: nowrap; }
.robot-inline-bar { flex: 1; height: 6px; background: #e0e0e0; border-radius: 3px; overflow: hidden; min-width: 80px; }
.robot-inline-fill { height: 100%; background: #2980b9; border-radius: 3px; transition: width 1s linear; }
.robot-inline-fill.no-transition { transition: none; }
.robot-inline-time { font-size: 0.75rem; color: #888; white-space: nowrap; }

/* ===== Links ===== */
.card-title-link { color: inherit; text-decoration: none; }
.card-title-link:hover { text-decoration: underline; }
.no-activity { color: #bbb; font-style: italic; font-size: 0.85rem; margin: 0; }
.barcode-link { text-decoration: none; font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace; color: #1a1a2e; font-weight: 600; }
.barcode-link:hover { text-decoration: underline; color: #2980b9; }
</style>
