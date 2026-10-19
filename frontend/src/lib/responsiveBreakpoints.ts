export const BREAKPOINTS = {
  xs: 0,
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  '2xl': 1536,
} as const

export type Breakpoint = keyof typeof BREAKPOINTS

export function getBreakpointValue(breakpoint: Breakpoint): number {
  return BREAKPOINTS[breakpoint]
}

export function getBreakpointPixels(breakpoint: Breakpoint): string {
  return `${getBreakpointValue(breakpoint)}px`
}

export function getBreakpointRem(breakpoint: Breakpoint): string {
  return `${getBreakpointValue(breakpoint) / 16}rem`
}

export function createMediaQuery(breakpoint: Breakpoint, direction: 'min' | 'max' = 'min'): string {
  const value = getBreakpointPixels(breakpoint)
  return `@media (${direction}-width: ${value})`
}

export function getResponsiveValue<T>(
  values: Partial<Record<Breakpoint, T>>,
  currentBreakpoint: Breakpoint
): T | undefined {
  return values[currentBreakpoint]
}

export function isMobileOnly(): boolean {
  return typeof window !== 'undefined' && window.innerWidth < getBreakpointValue('md')
}

export function isTabletAndUp(): boolean {
  return typeof window !== 'undefined' && window.innerWidth >= getBreakpointValue('md')
}

export function isDesktopAndUp(): boolean {
  return typeof window !== 'undefined' && window.innerWidth >= getBreakpointValue('lg')
}

export function getCurrentBreakpoint(): Breakpoint {
  if (typeof window === 'undefined') return 'xs'

  const width = window.innerWidth
  if (width >= BREAKPOINTS['2xl']) return '2xl'
  if (width >= BREAKPOINTS.xl) return 'xl'
  if (width >= BREAKPOINTS.lg) return 'lg'
  if (width >= BREAKPOINTS.md) return 'md'
  if (width >= BREAKPOINTS.sm) return 'sm'
  return 'xs'
}
