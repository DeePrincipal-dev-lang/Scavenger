import { useState, useRef, useCallback } from 'react'
import { Upload, Download, CheckCircle2, XCircle, AlertCircle, FileText, Loader2 } from 'lucide-react'
import { useAppTitle } from '@/hooks/useAppTitle'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/Card'
import { Button } from '@/components/ui/Button'
import { Badge } from '@/components/ui/Badge'
import { cn } from '@/lib/utils'
import { WasteType } from '@/api/types'

// CSV column order: waste_type, weight, latitude, longitude
const CSV_TEMPLATE = `waste_type,weight,latitude,longitude
Paper,10.5,40.7128,-74.0060
PetPlastic,5.0,40.7128,-74.0060
Plastic,8.3,40.7128,-74.0060
Metal,15.0,40.7128,-74.0060
Glass,6.2,40.7128,-74.0060
Organic,12.0,40.7128,-74.0060
Electronic,3.5,40.7128,-74.0060`

const WASTE_TYPE_MAP: Record<string, WasteType> = {
  Paper: WasteType.Paper,
  PetPlastic: WasteType.PetPlastic,
  Plastic: WasteType.Plastic,
  Metal: WasteType.Metal,
  Glass: WasteType.Glass,
  Organic: WasteType.Organic,
  Electronic: WasteType.Electronic,
}

interface ParsedRow {
  rowIndex: number
  waste_type: string
  weight: string
  latitude: string
  longitude: string
  errors: string[]
  valid: boolean
}

type SubmissionStatus = 'idle' | 'uploading' | 'done'

interface RowResult {
  rowIndex: number
  success: boolean
  message: string
}

function parseCSV(text: string): ParsedRow[] {
  const lines = text.trim().split('\n').filter(Boolean)
  if (lines.length < 2) return []
  const rows = lines.slice(1) // skip header

  return rows.map((line, idx) => {
    const [waste_type = '', weight = '', latitude = '', longitude = ''] = line
      .split(',')
      .map((v) => v.trim())

    const errors: string[] = []
    if (!WASTE_TYPE_MAP[waste_type]) errors.push(`Unknown waste type "${waste_type}"`)
    const w = parseFloat(weight)
    if (isNaN(w) || w <= 0) errors.push('Weight must be a positive number')
    const lat = parseFloat(latitude)
    if (isNaN(lat) || lat < -90 || lat > 90) errors.push('Latitude must be between -90 and 90')
    const lon = parseFloat(longitude)
    if (isNaN(lon) || lon < -180 || lon > 180) errors.push('Longitude must be between -180 and 180')

    return { rowIndex: idx + 2, waste_type, weight, latitude, longitude, errors, valid: errors.length === 0 }
  })
}

function downloadTemplate() {
  const blob = new Blob([CSV_TEMPLATE], { type: 'text/csv' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = 'batch_upload_template.csv'
  a.click()
  URL.revokeObjectURL(url)
}

export function BatchUploadPage() {
  useAppTitle('Batch Upload')
  const fileRef = useRef<HTMLInputElement>(null)
  const [rows, setRows] = useState<ParsedRow[]>([])
  const [fileName, setFileName] = useState('')
  const [status, setStatus] = useState<SubmissionStatus>('idle')
  const [progress, setProgress] = useState(0)
  const [results, setResults] = useState<RowResult[]>([])
  const [dragOver, setDragOver] = useState(false)

  const validRows = rows.filter((r) => r.valid)
  const invalidRows = rows.filter((r) => !r.valid)

  const handleFile = useCallback((file: File) => {
    setFileName(file.name)
    setStatus('idle')
    setResults([])
    const reader = new FileReader()
    reader.onload = (e) => {
      const text = e.target?.result as string
      setRows(parseCSV(text))
    }
    reader.readAsText(file)
  }, [])

  function onFileChange(e: React.ChangeEvent<HTMLInputElement>) {
    const file = e.target.files?.[0]
    if (file) handleFile(file)
  }

  function onDrop(e: React.DragEvent) {
    e.preventDefault()
    setDragOver(false)
    const file = e.dataTransfer.files[0]
    if (file && file.name.endsWith('.csv')) handleFile(file)
  }

  async function handleSubmit() {
    if (validRows.length === 0) return
    setStatus('uploading')
    setProgress(0)
    const res: RowResult[] = []

    for (let i = 0; i < validRows.length; i++) {
      // Simulated submission — replace with actual contract call
      await new Promise((r) => setTimeout(r, 200))
      res.push({ rowIndex: validRows[i].rowIndex, success: true, message: 'Submitted' })
      setProgress(Math.round(((i + 1) / validRows.length) * 100))
    }

    setResults(res)
    setStatus('done')
  }

  const succeeded = results.filter((r) => r.success).length
  const failed = results.filter((r) => !r.success).length

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">Batch Upload</h1>
        <p className="text-sm text-muted-foreground">Submit multiple waste items at once via CSV</p>
      </div>

      <div className="grid gap-6 lg:grid-cols-3">
        <div className="lg:col-span-2 space-y-4">
          {/* Drop zone */}
          <Card>
            <CardContent className="pt-6">
              <div
                role="button"
                tabIndex={0}
                aria-label="Drop CSV file or click to upload"
                onDragOver={(e) => { e.preventDefault(); setDragOver(true) }}
                onDragLeave={() => setDragOver(false)}
                onDrop={onDrop}
                onClick={() => fileRef.current?.click()}
                onKeyDown={(e) => e.key === 'Enter' && fileRef.current?.click()}
                className={cn(
                  'flex flex-col items-center justify-center gap-3 rounded-lg border-2 border-dashed p-10 text-center cursor-pointer transition-colors',
                  dragOver ? 'border-primary bg-accent' : 'border-muted-foreground/30 hover:border-primary hover:bg-accent/30'
                )}
              >
                <Upload className="h-8 w-8 text-muted-foreground" />
                <div>
                  <p className="font-medium">{fileName || 'Drop your CSV here or click to browse'}</p>
                  <p className="text-xs text-muted-foreground mt-1">Supports .csv files only</p>
                </div>
                <input
                  ref={fileRef}
                  type="file"
                  accept=".csv"
                  className="hidden"
                  onChange={onFileChange}
                  aria-label="Upload CSV file"
                />
              </div>
            </CardContent>
          </Card>

          {/* Preview table */}
          {rows.length > 0 && (
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-base flex items-center justify-between">
                  <span>Preview ({rows.length} rows)</span>
                  <div className="flex gap-2 text-xs font-normal">
                    <span className="text-green-600">{validRows.length} valid</span>
                    {invalidRows.length > 0 && (
                      <span className="text-destructive">{invalidRows.length} invalid</span>
                    )}
                  </div>
                </CardTitle>
              </CardHeader>
              <CardContent className="p-0">
                <div className="overflow-x-auto">
                  <table className="w-full text-sm">
                    <thead>
                      <tr className="border-b bg-muted/40">
                        <th className="px-4 py-2 text-left font-medium text-muted-foreground">Row</th>
                        <th className="px-4 py-2 text-left font-medium text-muted-foreground">Type</th>
                        <th className="px-4 py-2 text-left font-medium text-muted-foreground">Weight (kg)</th>
                        <th className="px-4 py-2 text-left font-medium text-muted-foreground">Lat</th>
                        <th className="px-4 py-2 text-left font-medium text-muted-foreground">Lon</th>
                        <th className="px-4 py-2 text-left font-medium text-muted-foreground">Status</th>
                      </tr>
                    </thead>
                    <tbody>
                      {rows.map((row) => (
                        <tr key={row.rowIndex} className={cn('border-b last:border-0', !row.valid && 'bg-destructive/5')}>
                          <td className="px-4 py-2 text-muted-foreground">{row.rowIndex}</td>
                          <td className="px-4 py-2">{row.waste_type}</td>
                          <td className="px-4 py-2">{row.weight}</td>
                          <td className="px-4 py-2">{row.latitude}</td>
                          <td className="px-4 py-2">{row.longitude}</td>
                          <td className="px-4 py-2">
                            {row.valid ? (
                              <CheckCircle2 className="h-4 w-4 text-green-500" />
                            ) : (
                              <span className="flex items-center gap-1 text-destructive text-xs">
                                <XCircle className="h-4 w-4 shrink-0" />
                                {row.errors[0]}
                              </span>
                            )}
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </CardContent>
            </Card>
          )}

          {/* Progress */}
          {status === 'uploading' && (
            <Card>
              <CardContent className="pt-6 space-y-3">
                <div className="flex items-center gap-2">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  <span className="text-sm font-medium">Submitting… {progress}%</span>
                </div>
                <div className="h-2 w-full overflow-hidden rounded-full bg-muted">
                  <div
                    className="h-full bg-primary transition-all duration-300"
                    style={{ width: `${progress}%` }}
                  />
                </div>
              </CardContent>
            </Card>
          )}

          {/* Results */}
          {status === 'done' && (
            <Card>
              <CardContent className="pt-6 space-y-3">
                <div className="flex items-center gap-2">
                  {failed === 0 ? (
                    <CheckCircle2 className="h-5 w-5 text-green-500" />
                  ) : (
                    <AlertCircle className="h-5 w-5 text-yellow-500" />
                  )}
                  <span className="font-medium text-sm">
                    {succeeded} submitted, {failed} failed
                  </span>
                </div>
                {failed > 0 && (
                  <ul className="text-xs text-destructive space-y-1">
                    {results.filter((r) => !r.success).map((r) => (
                      <li key={r.rowIndex}>Row {r.rowIndex}: {r.message}</li>
                    ))}
                  </ul>
                )}
              </CardContent>
            </Card>
          )}
        </div>

        {/* Sidebar */}
        <div className="space-y-4">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-base">Actions</CardTitle>
            </CardHeader>
            <CardContent className="space-y-3">
              <Button variant="outline" className="w-full justify-start" onClick={downloadTemplate}>
                <Download className="h-4 w-4 mr-2" />
                Download CSV Template
              </Button>
              <Button
                className="w-full"
                disabled={validRows.length === 0 || status === 'uploading'}
                onClick={() => void handleSubmit()}
              >
                {status === 'uploading' ? (
                  <><Loader2 className="h-4 w-4 mr-2 animate-spin" />Uploading…</>
                ) : (
                  <><Upload className="h-4 w-4 mr-2" />Submit {validRows.length} items</>
                )}
              </Button>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="flex items-center gap-2 text-base">
                <FileText className="h-4 w-4" />
                CSV Format
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-2 text-sm">
              <p className="text-muted-foreground text-xs">Required columns in order:</p>
              {['waste_type', 'weight', 'latitude', 'longitude'].map((col) => (
                <div key={col} className="flex items-center justify-between">
                  <code className="text-xs bg-muted px-1.5 py-0.5 rounded">{col}</code>
                  {col === 'waste_type' && (
                    <span className="text-xs text-muted-foreground">string</span>
                  )}
                  {col === 'weight' && (
                    <span className="text-xs text-muted-foreground">kg</span>
                  )}
                  {(col === 'latitude' || col === 'longitude') && (
                    <span className="text-xs text-muted-foreground">decimal</span>
                  )}
                </div>
              ))}
              <div className="mt-3 pt-3 border-t">
                <p className="text-xs text-muted-foreground mb-1.5">Valid waste types:</p>
                <div className="flex flex-wrap gap-1">
                  {Object.keys(WASTE_TYPE_MAP).map((t) => (
                    <Badge key={t} variant="outline" className="text-[10px]">{t}</Badge>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  )
}
