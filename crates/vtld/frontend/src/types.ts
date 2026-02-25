// Shared TypeScript interfaces for the QuantumVTL admin UI.

import type { ScsiLogSummary } from './api'

export interface ChangerDetail {
  vendor: string
  product: string
  serial: string
  firmware_version: string
  state: string
  temperature_c: number
  humidity_pct: number
  total_moves: number
  picker_position: number
  active_alerts: number[]
  prevent_medium_removal: boolean
  num_drives: number
  num_slots: number
  num_import_export: number
  elements: ElementDetail[]
}

export interface ElementDetail {
  address: number
  element_type: string
  full: boolean
  barcode: string | null
  source_element: number
  access: boolean
  except: boolean
  disabled: boolean
  asc_ascq: [number, number] | null
  medium_type: string
  import_export: boolean
  operator_intervention: boolean
}

export interface DriveSummary {
  id: number
  status: string
  serial: string
  barcode: string | null
  position: number
  record_count: number
}

export interface DriveDetail {
  id: number
  serial: string
  generation: string
  loaded: boolean
  barcode: string | null
  write_protected: boolean
  worm: boolean
  partition: number
  block_number: number
  file_number: number
  at_bop: boolean
  at_eod: boolean
  current_wrap: number | null
  total_wraps: number | null
  position_in_wrap_pct: number | null
  write_buffer_pct: number
  read_cache_pct: number
  objects_in_buffer: number
  buffer_state: string
  drive_state: string
  tape_speed: number | null
  operation_progress_pct: number | null
  instantaneous_rate_bytes_sec: number | null
  compression_ratio: number | null
  backhitch_count_this_mount: number
  capacity_used_pct: number | null
  native_bytes_written: number
  compressed_bytes_written: number
  approximate_remaining_mb: number | null
  total_loads: number
  motion_hours: number
}

export interface ConnectionInfo {
  cid: number
  peer_addr: string
  connected_since: string
  rx_commands: number
  rx_bytes: number
  tx_commands: number
  tx_bytes: number
  active_commands: number
  scsi_log: ScsiLogSummary[]
}

export interface SessionInfo {
  initiator_name: string
  tsih: number
  connections: ConnectionInfo[]
  active_commands: number
}
