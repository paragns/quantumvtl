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
