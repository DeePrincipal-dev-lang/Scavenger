import { useState } from 'react'
import { Flag, RotateCcw, ChevronDown, ChevronUp, BarChart2, Info } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/Card'
import { Badge } from '@/components/ui/Badge'
import { useFeatureFlags } from '@/hooks/useFeatureFlags'
import { getFlagAnalytics, type FeatureFlag } from '@/lib/featureFlags'

function FlagRow({ flag, value, hasOverride, onToggle, onClear }: {
  flag: FeatureFlag
  value: unknown
  hasOverride: boolean
  onToggle: (v: boolean) => void
  onClear: () => void
}) {
  const isOn = Boolean(value)

  return (
    <div className="flex items-start justify-between gap-4 py-4 border-b last:border-0">
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 flex-wrap">
          <span className="font-mono text-sm font-medium">{flag.key}</span>
          {hasOverride && (
            <Badge className="text-xs bg-yellow-100 text-yellow-700 border border-yellow-200 dark:bg-yellow-900/30 dark:text-yellow-400">
              overridden
            </Badge>
          )}
          {flag.rolloutPercentage !== undefined && (
            <Badge className="text-xs bg-blue-100 text-blue-700 border border-blue-200 dark:bg-blue-900/30 dark:text-blue-400">
              {flag.rolloutPercentage}% rollout
            </Badge>
          )}
        </div>
        <p className="text-sm text-muted-foreground mt-0.5">{flag.description}</p>
        {flag.environments && !flag.environments.includes('all') && (
          <p className="text-xs text-muted-foreground mt-0.5">
            Environments: {flag.environments.join(', ')}
          </p>
        )}
      </div>
      <div className="flex items-center gap-2 shrink-0">
        {hasOverride && (
          <button
            onClick={onClear}
            className="text-xs text-muted-foreground hover:text-foreground underline"
            title="Clear override"
          >
            reset
          </button>
        )}
        <button
          role="switch"
          aria-checked={isOn}
          onClick={() => onToggle(!isOn)}
          className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus-visible:outline focus-visible:outline-2 focus-visible:outline-ring ${
            isOn ? 'bg-green-500' : 'bg-gray-300 dark:bg-gray-600'
          }`}
        >
          <span
            className={`inline-block h-4 w-4 transform rounded-full bg-white shadow transition-transform ${
              isOn ? 'translate-x-6' : 'translate-x-1'
            }`}
          />
        </button>
      </div>
    </div>
  )
}

export function FeatureFlagsPage() {
  const { flags, values, overrides, override, clearOverride, clearAll } = useFeatureFlags()
  const [showAnalytics, setShowAnalytics] = useState(false)
  const analytics = getFlagAnalytics().slice(-20).reverse()

  const enabledCount = Object.values(values).filter(Boolean).length
  const overrideCount = overrides.length

  return (
    <main className="p-4 md:p-6 space-y-6 max-w-3xl mx-auto">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold flex items-center gap-2">
            <Flag className="h-6 w-6" />
            Feature Flags
          </h1>
          <p className="text-muted-foreground text-sm mt-1">
            Manage gradual rollouts, A/B tests, and environment-specific features.
          </p>
        </div>
        {overrideCount > 0 && (
          <button
            onClick={clearAll}
            className="flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
          >
            <RotateCcw className="h-4 w-4" />
            Reset all
          </button>
        )}
      </div>

      {/* Summary */}
      <div className="grid grid-cols-3 gap-4">
        <Card>
          <CardContent className="pt-4">
            <p className="text-2xl font-bold">{Object.keys(flags).length}</p>
            <p className="text-sm text-muted-foreground">Total flags</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <p className="text-2xl font-bold text-green-600">{enabledCount}</p>
            <p className="text-sm text-muted-foreground">Enabled</p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-4">
            <p className="text-2xl font-bold text-yellow-600">{overrideCount}</p>
            <p className="text-sm text-muted-foreground">Overridden</p>
          </CardContent>
        </Card>
      </div>

      {/* Flag list */}
      <Card>
        <CardHeader>
          <CardTitle>All Flags</CardTitle>
          <CardDescription>
            Toggles here create local overrides stored in your browser. Production changes
            require a deployment configuration update.
          </CardDescription>
        </CardHeader>
        <CardContent className="divide-y p-0 px-6">
          {Object.values(flags).map((flag) => (
            <FlagRow
              key={flag.key}
              flag={flag}
              value={values[flag.key]}
              hasOverride={overrides.some((o) => o.key === flag.key)}
              onToggle={(v) => override(flag.key, v)}
              onClear={() => clearOverride(flag.key)}
            />
          ))}
        </CardContent>
      </Card>

      {/* Analytics */}
      <Card>
        <CardHeader>
          <button
            className="flex items-center justify-between w-full text-left"
            onClick={() => setShowAnalytics((v) => !v)}
          >
            <CardTitle className="flex items-center gap-2">
              <BarChart2 className="h-4 w-4" />
              Recent Flag Events
            </CardTitle>
            {showAnalytics ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
          </button>
        </CardHeader>
        {showAnalytics && (
          <CardContent>
            {analytics.length === 0 ? (
              <p className="text-sm text-muted-foreground">No flag events recorded yet.</p>
            ) : (
              <ul className="space-y-1 text-sm">
                {analytics.map((e, i) => (
                  <li key={i} className="flex items-center gap-2 text-muted-foreground">
                    <span className="font-mono">{e.flagKey}</span>
                    <span>→</span>
                    <Badge className={`text-xs ${e.value ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'}`}>
                      {String(e.value)}
                    </Badge>
                    <span className="ml-auto text-xs">{new Date(e.timestamp).toLocaleTimeString()}</span>
                  </li>
                ))}
              </ul>
            )}
          </CardContent>
        )}
      </Card>

      {/* Info callout */}
      <div className="flex gap-2 p-3 rounded-lg bg-blue-50 dark:bg-blue-900/20 text-sm text-blue-700 dark:text-blue-300">
        <Info className="h-4 w-4 mt-0.5 shrink-0" />
        <p>
          Overrides apply only to this browser session. For permanent rollout changes, update{' '}
          <code className="font-mono text-xs">frontend/src/lib/featureFlags.ts</code> and redeploy.
        </p>
      </div>
    </main>
  )
}
