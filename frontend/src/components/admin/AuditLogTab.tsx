import { useState } from 'react'
import { formatDate } from '@/lib/helpers'
import { getAuditLog, type AuditEntry } from './auditLog'

export function AuditLogTab() {
  const [log] = useState<AuditEntry[]>(getAuditLog())

  return (
    <div className="space-y-2">
      {log.length === 0 ? (
        <p className="text-sm text-muted-foreground">No admin actions recorded this session.</p>
      ) : (
        <div className="divide-y divide-border rounded-lg border">
          {log.map((entry) => (
            <div key={entry.id} className="flex items-center justify-between px-4 py-3 text-sm">
              <div>
                <p className="font-medium">{entry.action}</p>
                <p className="text-xs text-muted-foreground">Target: {entry.target}</p>
              </div>
              <span className="text-xs text-muted-foreground">{formatDate(entry.timestamp)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
