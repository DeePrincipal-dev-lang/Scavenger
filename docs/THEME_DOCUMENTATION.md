# Theme System Documentation

## Overview

Theme logic (light/dark/high-contrast/system mode, toggling, and persistence) is
centralized in a single `ThemeProvider`. There is one source of truth for theme
state — no other component or hook should read/write theme preferences directly.

## Architecture

### ThemeProvider (`frontend/src/context/ThemeProvider.tsx`)

- Wraps [`next-themes`](https://github.com/pacocoursey/next-themes) to apply the
  resolved theme as a class on `<html>` and persist the user's choice to
  `localStorage` (key: `scavngr-theme`).
- Exposes a `useTheme()` hook with:
  - `theme`: `'light' | 'dark' | 'high-contrast' | 'system'`
  - `resolvedTheme`: `'light' | 'dark' | 'high-contrast'`
  - `isDark`, `isHighContrast`, `isReady`
  - `setTheme(theme)`, `toggleTheme()`
- Mounted once at the app root in `frontend/src/main.tsx`, wrapping the entire
  application so every component can call `useTheme()`.

### UI Components (`frontend/src/components/ui/ThemeToggle.tsx`)

- `ThemeToggle` — icon button that toggles between light/dark.
- `ThemeSelector` — segmented control for light / dark / system, used in
  `SettingsPage`.

## Usage

```typescript
import { useTheme } from '@/context/ThemeProvider'

function MyComponent() {
  const { theme, isDark, setTheme, toggleTheme } = useTheme()

  return (
    <button onClick={toggleTheme}>
      {isDark ? 'Switch to light' : 'Switch to dark'}
    </button>
  )
}
```

## Persistence

Theme choice is persisted automatically by `next-themes` to `localStorage`
under `scavngr-theme` and restored on load, before first paint, to avoid a
flash of incorrect theme.

## Accessibility

A `high-contrast` theme is supported alongside light/dark for WCAG-focused
users, toggled via the same `setTheme`/`useTheme` API.
