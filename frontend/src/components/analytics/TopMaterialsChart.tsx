import { Medal } from 'lucide-react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { TOP_MATERIALS, maxValue, percentOfMax, type MaterialVolume } from '@/lib/analytics'

interface TopMaterialsChartProps {
  materials?: MaterialVolume[]
}

export function TopMaterialsChart({ materials = TOP_MATERIALS }: TopMaterialsChartProps) {
  const max = maxValue(materials)
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-base">
          <Medal className="h-4 w-4" />
          Top Materials by Volume
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        {materials.map(({ label, value, color }, i) => (
          <div key={label} className="space-y-1">
            <div className="flex items-center justify-between text-sm">
              <span className="flex items-center gap-1.5">
                <span className="text-muted-foreground text-xs w-4">#{i + 1}</span>
                {label}
              </span>
              <span className="font-medium text-xs">{value} t</span>
            </div>
            <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
              <div
                className={`h-full ${color} transition-all duration-500`}
                style={{ width: `${percentOfMax(value, max)}%` }}
              />
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
