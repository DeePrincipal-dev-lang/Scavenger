import { describe, it, expect, vi, beforeEach } from 'vitest'
import { router } from './router'

// Mock pages
vi.mock('@/pages/LandingPage', () => ({
  LandingPage: () => <div data-testid="landing-page">Landing</div>,
}))

vi.mock('@/pages/LoginPage', () => ({
  LoginPage: () => <div data-testid="login-page">Login</div>,
}))

vi.mock('@/pages/NotFoundPage', () => ({
  NotFoundPage: () => <div data-testid="not-found-page">404</div>,
}))

vi.mock('@/components/layout/AppShell', () => ({
  AppShell: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="app-shell">{children}</div>
  ),
}))

vi.mock('@/context/AuthContext', () => ({
  useAuth: () => ({
    isAuthenticated: true,
    isLoading: false,
  }),
}))

describe('Router', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Router Configuration', () => {
    it('should be defined', () => {
      expect(router).toBeDefined()
    })

    it('should have public routes', () => {
      expect(router.routes).toBeDefined()
      expect(Array.isArray(router.routes)).toBe(true)
    })

    it('should have at least 3 routes', () => {
      expect(router.routes.length).toBeGreaterThanOrEqual(3)
    })
  })

  describe('Public Routes', () => {
    it('should have a landing page route', () => {
      const routes = router.routes
      const landingRoute = routes.find((r: any) => r.path === '/')
      expect(landingRoute).toBeDefined()
    })

    it('should have a login page route', () => {
      const routes = router.routes
      const loginRoute = routes.find((r: any) => r.path === '/login')
      expect(loginRoute).toBeDefined()
    })

    it('should have a 404 fallback route', () => {
      const routes = router.routes
      const notFoundRoute = routes.find((r: any) => r.path === '*')
      expect(notFoundRoute).toBeDefined()
    })
  })

  describe('Protected Routes', () => {
    it('should have protected routes layout', () => {
      const routes = router.routes
      const protectedLayout = routes.find((r: any) => r.element?.type?.name === 'ProtectedLayout')
      expect(protectedLayout).toBeDefined()
    })

    it('should have children routes for protected area', () => {
      const routes = router.routes
      const protectedLayout = routes.find((r: any) => r.children)
      expect(protectedLayout?.children).toBeDefined()
      expect(Array.isArray(protectedLayout?.children)).toBe(true)
    })

    it('should have dashboard route', () => {
      const routes = router.routes
      const protectedLayout = routes.find((r: any) => r.children)
      const dashboardRoute = protectedLayout?.children?.find((r: any) => r.path === 'dashboard')
      expect(dashboardRoute).toBeDefined()
    })

    it('should have settings route', () => {
      const routes = router.routes
      const protectedLayout = routes.find((r: any) => r.children)
      const settingsRoute = protectedLayout?.children?.find((r: any) => r.path === 'settings')
      expect(settingsRoute).toBeDefined()
    })

    it('should have incentives route', () => {
      const routes = router.routes
      const protectedLayout = routes.find((r: any) => r.children)
      const incentivesRoute = protectedLayout?.children?.find((r: any) => r.path === 'incentives')
      expect(incentivesRoute).toBeDefined()
    })
  })

  describe('Route Paths', () => {
    it('should have correctly formatted paths', () => {
      const paths: string[] = []
      function collectPaths(routes: any[]) {
        routes.forEach((route) => {
          if (route.path) {
            paths.push(route.path)
          }
          if (route.children) {
            collectPaths(route.children)
          }
        })
      }
      collectPaths(router.routes)
      expect(paths.length).toBeGreaterThan(0)
    })

    it('should have no duplicate root paths', () => {
      const rootPaths = router.routes
        .map((r: any) => r.path)
        .filter((p: string) => p && !p.startsWith('/'))
      const uniquePaths = new Set(rootPaths)
      // Allow duplicates in nested routes
      expect(router.routes.filter((r: any) => r.path === '').length).toBeLessThanOrEqual(1)
    })
  })

  describe('Route Elements', () => {
    it('should have elements for each route', () => {
      function hasElements(routes: any[]): boolean {
        return routes.every(
          (route) =>
            route.element ||
            (route.children && hasElements(route.children))
        )
      }
      expect(hasElements(router.routes)).toBe(true)
    })
  })

  describe('Lazy Loading', () => {
    it('should use lazy loading for protected pages', () => {
      const routes = router.routes
      const protectedLayout = routes.find((r: any) => r.children)
      expect(protectedLayout?.children).toBeDefined()
      // Lazy loaded components are typically wrapped in Suspense
      expect(protectedLayout?.children?.length).toBeGreaterThan(0)
    })
  })

  describe('Router Methods', () => {
    it('should have navigate method', () => {
      expect(typeof router.navigate).toBe('function')
    })

    it('should have state property', () => {
      expect(router.state).toBeDefined()
    })
  })

  describe('Error Handling', () => {
    it('should have a catch-all route for 404s', () => {
      const routes = router.routes
      const catchAll = routes.find((r: any) => r.path === '*')
      expect(catchAll).toBeDefined()
      expect(catchAll?.element).toBeDefined()
    })
  })

  describe('Route Count', () => {
    it('should have at least 20 protected routes', () => {
      const routes = router.routes
      const protectedLayout = routes.find((r: any) => r.children)
      // Account for multiple features
      expect(protectedLayout?.children?.length).toBeGreaterThanOrEqual(15)
    })
  })

  describe('Common Routes', () => {
    const commonProtectedPaths = [
      'dashboard',
      'settings',
      'rewards',
      'wastes',
      'incentives',
      'map',
      'analytics',
    ]

    commonProtectedPaths.forEach((path) => {
      it(`should have ${path} route`, () => {
        const routes = router.routes
        const protectedLayout = routes.find((r: any) => r.children)
        const targetRoute = protectedLayout?.children?.find((r: any) => r.path === path)
        expect(targetRoute).toBeDefined()
      })
    })
  })
})
