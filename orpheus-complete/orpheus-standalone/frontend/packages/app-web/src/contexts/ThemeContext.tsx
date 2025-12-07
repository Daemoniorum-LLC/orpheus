/**
 * Theme Context - Phthalo Green Yellow Shade (#0a3d2e)
 */
import { createContext, useCallback, useContext, useEffect, useMemo, useState, ReactNode } from 'react'
import { FluentProvider, Theme, createDarkTheme, createLightTheme, BrandVariants } from '@fluentui/react-components'

type ThemeMode = 'light' | 'dark' | 'system'

interface ThemeContextValue {
  mode: ThemeMode
  resolvedMode: 'dark' | 'light'
  theme: Theme
  toggleTheme: () => void
  setTheme: (mode: ThemeMode) => void
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined)
const STORAGE_KEY = 'orpheus-standalone-theme'

const phthaloBrand: BrandVariants = {
  10: '#020f0b', 20: '#041e17', 30: '#062d22', 40: '#0a3d2e',
  50: '#0d4c39', 60: '#115c45', 70: '#156b51', 80: '#1a7b5d',
  90: '#1f8a69', 100: '#259a76', 110: '#2caa83', 120: '#4db896',
  130: '#6ec6a9', 140: '#8fd4bc', 150: '#b0e2cf', 160: '#d1f0e2',
}

const darkTheme: Theme = createDarkTheme(phthaloBrand)
const lightTheme: Theme = createLightTheme(phthaloBrand)

const getSystemPref = () => window.matchMedia?.('(prefers-color-scheme: light)').matches ? 'light' : 'dark'

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [mode, setMode] = useState<ThemeMode>(() => {
    const s = localStorage.getItem(STORAGE_KEY)
    return (s === 'dark' || s === 'light' || s === 'system') ? s : 'dark'
  })

  const resolvedMode = useMemo(() => mode === 'system' ? getSystemPref() : mode, [mode])
  const theme = useMemo(() => resolvedMode === 'dark' ? darkTheme : lightTheme, [resolvedMode])

  const setTheme = useCallback((m: ThemeMode) => { setMode(m); localStorage.setItem(STORAGE_KEY, m) }, [])
  const toggleTheme = useCallback(() => setTheme(resolvedMode === 'light' ? 'dark' : 'light'), [resolvedMode, setTheme])

  useEffect(() => { document.documentElement.setAttribute('data-persona-theme', resolvedMode) }, [resolvedMode])

  return (
    <ThemeContext.Provider value={{ mode, resolvedMode, theme, toggleTheme, setTheme }}>
      <FluentProvider theme={theme}>{children}</FluentProvider>
    </ThemeContext.Provider>
  )
}

export const useTheme = () => {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider')
  return ctx
}
