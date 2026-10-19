import { useCallback } from 'react'
import { analytics, type EventCategory } from '@/lib/analyticsService'

/**
 * Hook that exposes event tracking helpers bound to an optional userId.
 */
export function useAnalytics(userId?: string) {
  const track = useCallback(
    (
      category: EventCategory,
      action: string,
      options: { label?: string; value?: number; metadata?: Record<string, unknown> } = {},
    ) => analytics.track(category, action, { ...options, userId }),
    [userId],
  )

  const pageView = useCallback(
    (page: string) => analytics.pageView(page, userId),
    [userId],
  )

  return { track, pageView }
}
