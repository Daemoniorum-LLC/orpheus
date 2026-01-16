/**
 * Theme Context - Phthalo Green Yellow Shade (#0a3d2e)
 */
import { createContext, useCallback, useContext, useEffect, useMemo, useState, ReactNode } from 'react'
import { ThemeProvider as UIThemeProvider } from '@persona-framework/ui'
import { safeStorage } from '../utils/storage'

type ThemeMode = 'light' | 'dark' | 'system'

interface ThemeContextValue {
  mode: ThemeMode
  resolvedMode: 'dark' | 'light'
  toggleTheme: () => void
  setTheme: (mode: ThemeMode) => void
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined)
const STORAGE_KEY = 'orpheus-theme'

const getSystemPref = () => window.matchMedia?.('(prefers-color-scheme: light)').matches ? 'light' : 'dark'

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [mode, setMode] = useState<ThemeMode>(() => {
    const s = safeStorage.getItem(STORAGE_KEY)
    return (s === 'dark' || s === 'light' || s === 'system') ? s : 'dark'
  })

  const resolvedMode = useMemo(() => mode === 'system' ? getSystemPref() : mode, [mode])

  const setTheme = useCallback((m: ThemeMode) => { setMode(m); safeStorage.setItem(STORAGE_KEY, m) }, [])
  const toggleTheme = useCallback(() => setTheme(resolvedMode === 'light' ? 'dark' : 'light'), [resolvedMode, setTheme])

  useEffect(() => { document.documentElement.setAttribute('data-persona-theme', resolvedMode) }, [resolvedMode])

  return (
    <ThemeContext.Provider value={{ mode, resolvedMode, toggleTheme, setTheme }}>
      <UIThemeProvider defaultTheme={resolvedMode} storageKey={STORAGE_KEY}>
        {children}
      </UIThemeProvider>
    </ThemeContext.Provider>
  )
}

export const useTheme = () => {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider')
  return ctx
}
