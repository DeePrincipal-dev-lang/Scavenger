// ── Audit log (local session only) ───────────────────────────────────────────

export interface AuditEntry {
  id: number
  action: string
  target: string
  timestamp: number
}

let _auditId = 0
const _auditLog: AuditEntry[] = []

export function addAuditEntry(action: string, target: string) {
  _auditLog.unshift({ id: ++_auditId, action, target, timestamp: Date.now() / 1000 })
  if (_auditLog.length > 50) _auditLog.pop()
}

export function getAuditLog(): AuditEntry[] {
  return [..._auditLog]
}
