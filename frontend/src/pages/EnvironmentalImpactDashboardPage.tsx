import { useState } from 'react'
import {
  Leaf,
  Zap,
  Droplets,
  Trees,
  Car,
  Smartphone,
  ShowerHead,
  Lightbulb,
  Share2,
  Check,
  Trophy,
  Target,
  TrendingUp,
} from 'lucide-react'
import { useImpactCalculator } from '@/hooks/useImpactCalculator'
import { useAppTitle } from '@/hooks/useAppTitle'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Badge } from '@/components/ui/Badge'
import { StatCard } from '@/components/ui/StatCard'
import { EmptyState } from '@/components/ui/EmptyState'
import { StatCardSkeleton } from '@/components/ui/Skeletons'

// ── Milestone definitions ────────────────────────────────────────────────────

interface Milestone {
  id: string
  label: string
  icon: string
  threshold: number
  unit: string
  getValue: (co2: number, energy: number, water: number) => number
}

const MILESTONES: Milestone[] = [
  { id: 'co2_10', label: '10 kg CO₂ Saved', icon: '🌿', threshold: 10, unit: 'kg CO₂', getValue: (co2) => co2 },
  { id: 'co2_100', label: '100 kg CO₂ Saved', icon: '🌳', threshold: 100, unit: 'kg CO₂', getValue: (co2) => co2 },
  { id: 'energy_50', label: '50 kWh Saved', icon: '⚡', threshold: 50, unit: 'kWh', getValue: (_, energy) => energy },
  { id: 'water_500', label: '500 L Water Saved', icon: '💧', threshold: 500, unit: 'litres', getValue: (_, __, water) => water },
  { id: 'co2_1000', label: 'Carbon Champion', icon: '🏆', threshold: 1000, unit: 'kg CO₂', getValue: (co2) => co2 },
]

// ── Sub-components ────────────────────────────────────────────────────────────

function ImpactMetric({
  icon,
  label,
  value,
  unit,
  colorClass,
}: {
  icon: React.ReactNode
  label: string
  value: number
  unit: string
  colorClass: string
}) {
  return (
    <Card>
      <CardContent className="flex flex-col items-center gap-2 py-6 text-center">
        <div className={`rounded-full p-3 ${colorClass}`}>{icon}</div>
        <p className="text-3xl font-bold">{value.toLocaleString()}</p>
        <p className="text-sm font-medium">{unit}</p>
        <p className="text-xs text-muted-foreground">{label}</p>
      </CardContent>
    </Card>
  )
}

function EquivalentRow({ icon, value, label }: { icon: React.ReactNode; value: number; label: string }) {
  return (
    <div className="flex items-center gap-3 rounded-md border px-4 py-3">
      <div className="text-muted-foreground">{icon}</div>
      <div>
        <span className="font-semibold">{value.toLocaleString()}</span>{' '}
        <span className="text-sm text-muted-foreground">{label}</span>
      </div>
    </div>
  )
}

function MilestoneBadge({ milestone, current }: { milestone: Milestone; current: number }) {
  const earned = current >= milestone.threshold
  const pct = Math.min(100, Math.round((current / milestone.threshold) * 100))
  return (
    <div className={`flex flex-col items-center gap-2 rounded-lg border p-4 text-center transition-opacity ${earned ? '' : 'opacity-50'}`}>
      <span className="text-3xl">{milestone.icon}</span>
      <p className="text-xs font-medium leading-tight">{milestone.label}</p>
      {earned ? (
        <Badge variant="success" className="text-xs">Earned</Badge>
      ) : (
        <div className="w-full">
          <div className="mb-1 h-1.5 w-full overflow-hidden rounded-full bg-muted">
            <div className="h-full rounded-full bg-primary" style={{ width: `${pct}%` }} />
          </div>
          <p className="text-[10px] text-muted-foreground">{pct}%</p>
        </div>
      )}
    </div>
  )
}

// ── Page ─────────────────────────────────────────────────────────────────────

export function EnvironmentalImpactDashboardPage() {
  useAppTitle('Environmental Impact Dashboard')
  const { impact, equivalents, shareText, isLoading, isError } = useImpactCalculator()
  const [copied, setCopied] = useState(false)

  const handleShare = async () => {
    if (navigator.share) {
      await navigator.share({ text: shareText }).catch(() => null)
    } else {
      await navigator.clipboard.writeText(shareText)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    }
  }

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
          {Array.from({ length: 4 }).map((_, i) => <StatCardSkeleton key={i} />)}
        </div>
      </div>
    )
  }

  if (isError) {
    return <EmptyState title="Failed to load" description="Could not fetch your waste data." icon={<Leaf className="h-8 w-8" />} />
  }

  const hasData = impact.co2Kg > 0 || impact.energyKwh > 0 || impact.waterLitres > 0

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h1 className="text-xl font-bold sm:text-2xl">Environmental Impact Dashboard</h1>
          <p className="text-sm text-muted-foreground">Track the positive impact of your recycling activities</p>
        </div>
        <Button size="sm" variant="outline" onClick={handleShare} disabled={!hasData}>
          {copied ? (
            <><Check className="mr-1 h-4 w-4 text-green-500" />Copied</>
          ) : (
            <><Share2 className="mr-1 h-4 w-4" />Share Impact</>
          )}
        </Button>
      </div>

      {!hasData ? (
        <EmptyState
          title="No impact data yet"
          description="Start recycling to see your environmental impact here."
          icon={<Leaf className="h-8 w-8 text-muted-foreground" />}
        />
      ) : (
        <>
          {/* Impact score summary */}
          <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
            <StatCard
              label="CO₂ Saved"
              value={`${impact.co2Kg} kg`}
              icon={<Leaf className="h-5 w-5 text-green-600" />}
            />
            <StatCard
              label="Energy Saved"
              value={`${impact.energyKwh} kWh`}
              icon={<Zap className="h-5 w-5 text-yellow-600" />}
            />
            <StatCard
              label="Water Saved"
              value={`${impact.waterLitres} L`}
              icon={<Droplets className="h-5 w-5 text-blue-600" />}
            />
            <StatCard
              label="Trees Equivalent"
              value={`${impact.treesEquivalent}`}
              icon={<Trees className="h-5 w-5 text-emerald-600" />}
            />
          </div>

          {/* Detailed metrics */}
          <div className="grid grid-cols-2 gap-4 sm:grid-cols-4">
            <ImpactMetric
              icon={<Leaf className="h-6 w-6 text-green-600" />}
              label="Carbon Saved"
              value={impact.co2Kg}
              unit="kg CO₂"
              colorClass="bg-green-100 dark:bg-green-900/30"
            />
            <ImpactMetric
              icon={<Zap className="h-6 w-6 text-yellow-600" />}
              label="Energy Saved"
              value={impact.energyKwh}
              unit="kWh"
              colorClass="bg-yellow-100 dark:bg-yellow-900/30"
            />
            <ImpactMetric
              icon={<Droplets className="h-6 w-6 text-blue-600" />}
              label="Water Saved"
              value={impact.waterLitres}
              unit="litres"
              colorClass="bg-blue-100 dark:bg-blue-900/30"
            />
            <ImpactMetric
              icon={<Trees className="h-6 w-6 text-emerald-600" />}
              label="Trees Equivalent"
              value={impact.treesEquivalent}
              unit="tree-years"
              colorClass="bg-emerald-100 dark:bg-emerald-900/30"
            />
          </div>

          {/* Everyday equivalents */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <TrendingUp className="h-4 w-4" />
                That's equivalent to…
              </CardTitle>
            </CardHeader>
            <CardContent className="grid gap-2 sm:grid-cols-2">
              <EquivalentRow icon={<Car className="h-4 w-4" />} value={equivalents.carKm} label="km not driven" />
              <EquivalentRow icon={<Smartphone className="h-4 w-4" />} value={equivalents.smartphoneCharges} label="smartphone charges" />
              <EquivalentRow icon={<ShowerHead className="h-4 w-4" />} value={equivalents.showerMinutes} label="minutes of showering saved" />
              <EquivalentRow icon={<Lightbulb className="h-4 w-4" />} value={equivalents.lightbulbHours} label="hours of LED lighting" />
            </CardContent>
          </Card>

          {/* Milestone badges */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Trophy className="h-4 w-4" />
                Impact Milestones
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-2 gap-3 sm:grid-cols-5">
                {MILESTONES.map((m) => (
                  <MilestoneBadge
                    key={m.id}
                    milestone={m}
                    current={m.getValue(impact.co2Kg, impact.energyKwh, impact.waterLitres)}
                  />
                ))}
              </div>
            </CardContent>
          </Card>

          {/* Impact goals */}
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2 text-base">
                <Target className="h-4 w-4" />
                Next Goal
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              {MILESTONES.filter(
                (m) => m.getValue(impact.co2Kg, impact.energyKwh, impact.waterLitres) < m.threshold
              )
                .slice(0, 1)
                .map((m) => {
                  const current = m.getValue(impact.co2Kg, impact.energyKwh, impact.waterLitres)
                  const pct = Math.min(100, Math.round((current / m.threshold) * 100))
                  return (
                    <div key={m.id} className="space-y-2">
                      <div className="flex items-center justify-between text-sm">
                        <span className="font-medium">{m.label}</span>
                        <span className="text-muted-foreground">
                          {current.toLocaleString()} / {m.threshold.toLocaleString()} {m.unit}
                        </span>
                      </div>
                      <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
                        <div
                          className="h-full rounded-full bg-primary transition-all"
                          style={{ width: `${pct}%` }}
                        />
                      </div>
                      <p className="text-xs text-muted-foreground">{pct}% complete</p>
                    </div>
                  )
                })}
              {MILESTONES.every(
                (m) => m.getValue(impact.co2Kg, impact.energyKwh, impact.waterLitres) >= m.threshold
              ) && (
                <p className="text-sm text-green-600 font-medium">🎉 All milestones achieved!</p>
              )}
            </CardContent>
          </Card>

          {/* Social sharing */}
          <Card>
            <CardHeader>
              <CardTitle className="text-base">Share your impact</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <p className="rounded-md bg-muted px-4 py-3 text-sm leading-relaxed">{shareText}</p>
              <Button onClick={handleShare} className="w-full sm:w-auto">
                {copied ? (
                  <><Check className="mr-1 h-4 w-4" />Copied to clipboard</>
                ) : (
                  <><Share2 className="mr-1 h-4 w-4" />Share on Social Media</>
                )}
              </Button>
            </CardContent>
          </Card>
        </>
      )}
    </div>
  )
}
