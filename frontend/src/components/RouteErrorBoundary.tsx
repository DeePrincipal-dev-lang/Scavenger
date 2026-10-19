import { isRouteErrorResponse, useNavigate, useRouteError } from 'react-router-dom'
import { Button } from '@/components/ui/Button'

// Vite's dynamic import() rejects with messages like this when a route chunk
// fails to load (e.g. a stale chunk hash after a new deployment).
const CHUNK_LOAD_ERROR_RE = /fetch dynamically imported module|importing a module script failed/i

/**
 * Per-route error boundary, wired via React Router's `errorElement`. Each
 * route gets its own instance, so a crash on one page shows a fallback in
 * place of just that route — the rest of the app shell and other routes are
 * unaffected, and navigating away recovers automatically.
 */
export function RouteErrorBoundary() {
  const error = useRouteError()
  const navigate = useNavigate()

  console.error('[RouteErrorBoundary]', error)

  const message = error instanceof Error ? error.message : String(error)
  const isChunkLoadError = error instanceof Error && CHUNK_LOAD_ERROR_RE.test(message)

  if (isChunkLoadError) {
    return (
      <div role="alert" className="flex min-h-[60vh] flex-col items-center justify-center gap-4 p-8 text-center">
        <p className="text-lg font-semibold">Failed to load this page.</p>
        <p className="max-w-sm text-sm text-muted-foreground">
          The page could not be loaded, possibly due to a network error or a new deployment.
          Reloading usually fixes it.
        </p>
        <Button onClick={() => window.location.reload()}>Reload page</Button>
      </div>
    )
  }

  const description = isRouteErrorResponse(error)
    ? error.statusText || `Error ${error.status}`
    : message || 'An unexpected error occurred.'

  return (
    <div role="alert" className="flex min-h-[60vh] flex-col items-center justify-center gap-4 p-8 text-center">
      <p className="text-lg font-semibold">Something went wrong on this page.</p>
      <p className="max-w-sm text-sm text-muted-foreground">{description}</p>
      <div className="flex gap-3">
        <Button variant="outline" onClick={() => navigate(0)}>Try again</Button>
        <Button onClick={() => navigate('/dashboard')}>Go to dashboard</Button>
      </div>
    </div>
  )
}
