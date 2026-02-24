export async function apiFetch(path: string, options: RequestInit = {}): Promise<Response> {
  const token = localStorage.getItem('token')
  const headers: Record<string, string> = {
    ...(options.headers as Record<string, string> || {}),
  }
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }
  if (options.body && typeof options.body === 'string') {
    headers['Content-Type'] = 'application/json'
  }
  return fetch(path, { ...options, headers })
}

export function isLoggedIn(): boolean {
  return !!localStorage.getItem('token')
}

export async function checkTokenValid(): Promise<boolean> {
  const token = localStorage.getItem('token')
  if (!token) return false
  try {
    const resp = await apiFetch('/api/vtl/status')
    return resp.ok
  } catch {
    return false
  }
}

// --- SCSI Log Types ---

export interface ScsiLogSummary {
  seq: number
  timestamp: string
  duration_us: number
  opcode: number
  opcode_name: string
  status: number
  status_name: string
  data_out_len: number
  data_in_len: number
  has_sense: boolean
}

export interface ScsiLogResponse {
  device_type: string
  device_id: number
  entries: ScsiLogSummary[]
}

export interface CdbField {
  name: string
  byte_offset: number
  bit_range: string | null
  hex_value: string
  decoded: string
}

export interface CdbBreakdown {
  opcode: number
  opcode_name: string
  cdb_length: number
  fields: CdbField[]
  hex_dump: string
}

export interface SenseBreakdown {
  sense_key: number
  sense_key_name: string
  asc: number
  ascq: number
  asc_description: string
  hex_dump: string
}

export interface ResponseBreakdown {
  status: number
  status_name: string
  data_in_length: number
  data_in_hex: string | null
  sense: SenseBreakdown | null
}

export interface ScsiCommandDetail {
  seq: number
  timestamp: string
  duration_us: number
  opcode: number
  opcode_name: string
  cdb_hex: string
  data_out_hex: string | null
  data_out_len: number
  status: number
  status_name: string
  data_in_hex: string | null
  data_in_len: number
  sense_hex: string
  cdb_breakdown: CdbBreakdown
  response_breakdown: ResponseBreakdown
}

// --- SCSI Log Fetch Functions ---

export async function fetchScsiLog(
  deviceType: string,
  deviceId: number,
  limit: number = 20
): Promise<ScsiLogResponse | null> {
  const path = deviceType === 'changer'
    ? `/api/vtl/scsi-log/changer?limit=${limit}`
    : `/api/vtl/scsi-log/drive/${deviceId}?limit=${limit}`
  try {
    const resp = await apiFetch(path)
    if (resp.ok) return await resp.json()
  } catch { /* ignore */ }
  return null
}

export async function fetchScsiLogEntry(
  deviceType: string,
  deviceId: number,
  seq: number
): Promise<ScsiCommandDetail | null> {
  try {
    const resp = await apiFetch(`/api/vtl/scsi-log/entry/${deviceType}/${deviceId}/${seq}`)
    if (resp.ok) return await resp.json()
  } catch { /* ignore */ }
  return null
}
