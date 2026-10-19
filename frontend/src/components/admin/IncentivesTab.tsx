import { useState } from 'react'
import { wasteTypeLabel, formatAddress } from '@/lib/helpers'
import { Badge } from '@/components/ui/Badge'
import { Input } from '@/components/ui/Input'
import { useAdminIncentives } from '@/hooks/useAdminDashboard'

export function IncentivesTab() {
  const { data: incentives = [], isLoading } = useAdminIncentives()
  const [filter, setFilter] = useState('')

  const filtered = incentives.filter(
    (inc) =>
      !filter ||
      inc.rewarder.toLowerCase().includes(filter.toLowerCase()) ||
      wasteTypeLabel(inc.waste_type).toLowerCase().includes(filter.toLowerCase())
  )

  return (
    <div className="space-y-4">
      <Input
        placeholder="Filter by rewarder or type…"
        value={filter}
        onChange={(e) => setFilter(e.target.value)}
        aria-label="Filter incentives"
      />
      {isLoading ? (
        <p className="text-sm text-muted-foreground">Loading…</p>
      ) : filtered.length === 0 ? (
        <p className="text-sm text-muted-foreground">No incentives found.</p>
      ) : (
        <div className="divide-y divide-border rounded-lg border">
          {filtered.map((inc) => (
            <div key={inc.id} className="flex items-center justify-between px-4 py-3 text-sm">
              <div>
                <p className="font-medium">
                  {wasteTypeLabel(inc.waste_type)}{' '}
                  <span className="text-muted-foreground">#{inc.id}</span>
                </p>
                <p className="text-xs text-muted-foreground">
                  {formatAddress(inc.rewarder)} · {inc.reward_points} pts · Budget:{' '}
                  {inc.remaining_budget}/{inc.total_budget}
                </p>
              </div>
              <Badge variant={inc.active ? 'default' : 'outline'}>
                {inc.active ? 'Active' : 'Inactive'}
              </Badge>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
