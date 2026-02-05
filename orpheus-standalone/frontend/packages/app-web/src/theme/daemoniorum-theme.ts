/**
 * Daemoniorum Corporate Goth Theme
 * A sophisticated dark theme with phthalo green accents
 *
 * Design Philosophy:
 * - Deep, near-black backgrounds for dramatic effect
 * - Phthalo green (#123524) as the primary accent color
 * - Subtle gray hierarchy for depth without distraction
 * - Clean, professional typography with gothic undertones
 * - Matte finishes, no glossy effects
 */

import { createDarkTheme, createLightTheme, BrandVariants } from '@fluentui/react-components';

/**
 * Phthalo Green brand palette
 * Generated from base color #123524
 */
export const phthaloGreenPalette: BrandVariants = {
  10: '#020805',  // Deepest - near black with green tint
  20: '#061109',  // Very dark
  30: '#0a1a0e',  // Deep forest
  40: '#0d2314',  // Dark phthalo
  50: '#112c1a',  // Phthalo shadow
  60: '#153520',  // Core phthalo dark
  70: '#1a4a2c',  // Phthalo base
  80: '#1e5e38',  // Phthalo medium
  90: '#237245',  // Phthalo bright
  100: '#2a8752', // Phthalo light
  110: '#349c60', // Phthalo highlight
  120: '#42b170', // Lighter
  130: '#5bc284', // Light
  140: '#7ad39a', // Very light
  150: '#a3e4b8', // Pale
  160: '#d1f2da', // Palest
};

/**
 * Corporate Goth Dark Theme
 * Primary theme for Orpheus/Maestro
 */
export const daemoniorumDarkTheme = createDarkTheme(phthaloGreenPalette);

// Override specific tokens for the corporate goth aesthetic
Object.assign(daemoniorumDarkTheme, {
  // Background hierarchy - deep blacks and charcoals
  colorNeutralBackground1: '#0d0d0d',        // Main content background
  colorNeutralBackground1Hover: '#141414',
  colorNeutralBackground1Pressed: '#0a0a0a',
  colorNeutralBackground1Selected: '#1a1a1a',

  colorNeutralBackground2: '#111111',        // Secondary (toolbar)
  colorNeutralBackground2Hover: '#181818',
  colorNeutralBackground2Pressed: '#0d0d0d',
  colorNeutralBackground2Selected: '#1f1f1f',

  colorNeutralBackground3: '#161616',        // Tertiary (sidebar, cards)
  colorNeutralBackground3Hover: '#1c1c1c',
  colorNeutralBackground3Pressed: '#121212',

  colorNeutralBackground4: '#1a1a1a',        // Subtle elevated surfaces
  colorNeutralBackground5: '#202020',        // More elevated
  colorNeutralBackground6: '#262626',        // Highest elevation

  // Foreground colors - crisp whites and grays
  colorNeutralForeground1: '#f0f0f0',        // Primary text
  colorNeutralForeground2: '#b8b8b8',        // Secondary text
  colorNeutralForeground3: '#808080',        // Tertiary/muted text
  colorNeutralForeground4: '#606060',        // Disabled text
  colorNeutralForegroundDisabled: '#4a4a4a',

  // Strokes/borders - subtle but visible
  colorNeutralStroke1: '#2a2a2a',            // Primary borders
  colorNeutralStroke2: '#363636',            // Secondary borders
  colorNeutralStroke3: '#424242',            // Subtle dividers
  colorNeutralStrokeAccessible: '#707070',   // Accessible borders
  colorNeutralStrokeDisabled: '#333333',

  // Brand colors - phthalo green with proper contrast
  colorBrandBackground: '#1a4a2c',           // Primary button bg
  colorBrandBackgroundHover: '#1e5e38',      // Hover state
  colorBrandBackgroundPressed: '#153520',    // Pressed state
  colorBrandBackgroundSelected: '#237245',   // Selected state

  colorBrandBackground2: '#0d2314',          // Subtle brand surface
  colorBrandBackground2Hover: '#112c1a',
  colorBrandBackground2Pressed: '#0a1a0e',

  colorBrandForeground1: '#5bc284',          // Brand text on dark
  colorBrandForeground2: '#7ad39a',          // Secondary brand text
  colorBrandForegroundLink: '#5bc284',       // Links
  colorBrandForegroundLinkHover: '#7ad39a',
  colorBrandForegroundLinkPressed: '#42b170',
  colorBrandForegroundLinkSelected: '#349c60',

  // On-brand colors (text on brand backgrounds)
  colorNeutralForegroundOnBrand: '#f0f0f0',
  colorNeutralForegroundInvertedLink: '#f0f0f0',

  // Status colors - muted to fit corporate goth
  colorPaletteRedBackground1: '#1a0d0d',
  colorPaletteRedBackground2: '#2a1414',
  colorPaletteRedBackground3: '#4a1e1e',
  colorPaletteRedForeground1: '#e06060',
  colorPaletteRedForeground2: '#cc4444',
  colorPaletteRedForeground3: '#b33030',

  colorPaletteGreenBackground1: '#0d1a10',
  colorPaletteGreenBackground2: '#142a1a',
  colorPaletteGreenBackground3: '#1e4a2a',
  colorPaletteGreenForeground1: '#60c080',
  colorPaletteGreenForeground2: '#4aa868',
  colorPaletteGreenForeground3: '#349050',

  colorPaletteYellowBackground1: '#1a1608',
  colorPaletteYellowBackground2: '#2a2410',
  colorPaletteYellowBackground3: '#4a3e18',
  colorPaletteYellowForeground1: '#d4b040',
  colorPaletteYellowForeground2: '#c0a030',
  colorPaletteYellowForeground3: '#a08020',

  colorPaletteDarkOrangeBackground3: '#3d2010',
  colorPaletteDarkOrangeForeground1: '#d08040',

  // Subtle backgrounds for UI elements
  colorSubtleBackground: 'transparent',
  colorSubtleBackgroundHover: '#1a1a1a',
  colorSubtleBackgroundPressed: '#141414',
  colorSubtleBackgroundSelected: '#202020',

  // Transparent backgrounds
  colorTransparentBackground: 'transparent',
  colorTransparentBackgroundHover: 'rgba(255,255,255,0.04)',
  colorTransparentBackgroundPressed: 'rgba(255,255,255,0.02)',
  colorTransparentBackgroundSelected: 'rgba(255,255,255,0.06)',

  // Shadows - subtle and matte
  shadow2: '0 1px 2px rgba(0,0,0,0.3)',
  shadow4: '0 2px 4px rgba(0,0,0,0.4)',
  shadow8: '0 4px 8px rgba(0,0,0,0.5)',
  shadow16: '0 8px 16px rgba(0,0,0,0.6)',
  shadow28: '0 14px 28px rgba(0,0,0,0.7)',
  shadow64: '0 32px 64px rgba(0,0,0,0.8)',

  // Border radius - slightly rounded for sophistication
  borderRadiusNone: '0',
  borderRadiusSmall: '2px',
  borderRadiusMedium: '4px',
  borderRadiusLarge: '6px',
  borderRadiusXLarge: '8px',
  borderRadiusCircular: '50%',
});

/**
 * CSS custom properties for additional theming
 * Use these for gradients and effects not covered by Fluent tokens
 */
export const daemoniorumCSSVars = {
  // Brand gradients - subtle and sophisticated
  '--daemoniorum-gradient-brand': 'linear-gradient(135deg, #1a4a2c 0%, #0d2314 100%)',
  '--daemoniorum-gradient-brand-hover': 'linear-gradient(135deg, #1e5e38 0%, #153520 100%)',
  '--daemoniorum-gradient-accent': 'linear-gradient(135deg, #1e5e38 0%, #1a4a2c 50%, #237245 100%)',

  // Surface gradients for depth
  '--daemoniorum-gradient-surface': 'linear-gradient(180deg, #161616 0%, #111111 100%)',
  '--daemoniorum-gradient-elevated': 'linear-gradient(180deg, #1a1a1a 0%, #141414 100%)',

  // Glow effects for focus states
  '--daemoniorum-glow-brand': '0 0 20px rgba(26, 74, 44, 0.4)',
  '--daemoniorum-glow-brand-strong': '0 0 30px rgba(42, 135, 82, 0.5)',

  // Phthalo green palette for direct use
  '--color-phthalo-darkest': '#0a1a0e',
  '--color-phthalo-dark': '#123524',
  '--color-phthalo-base': '#1a4a2c',
  '--color-phthalo-medium': '#1e5e38',
  '--color-phthalo-bright': '#237245',
  '--color-phthalo-light': '#2a8752',
  '--color-phthalo-highlight': '#5bc284',

  // Charcoal palette
  '--color-charcoal-900': '#0a0a0a',
  '--color-charcoal-850': '#0d0d0d',
  '--color-charcoal-800': '#111111',
  '--color-charcoal-750': '#141414',
  '--color-charcoal-700': '#161616',
  '--color-charcoal-650': '#1a1a1a',
  '--color-charcoal-600': '#202020',
  '--color-charcoal-500': '#2a2a2a',
  '--color-charcoal-400': '#363636',
  '--color-charcoal-300': '#4a4a4a',

  // Typography
  '--font-display': "'Inter', 'SF Pro Display', -apple-system, BlinkMacSystemFont, sans-serif",
  '--font-body': "'Inter', 'SF Pro Text', -apple-system, BlinkMacSystemFont, sans-serif",
  '--font-mono': "'JetBrains Mono', 'SF Mono', 'Fira Code', monospace",
};

/**
 * Light theme (if needed for accessibility toggle)
 * Maintains corporate goth feel with inverted contrast
 */
export const daemoniorumLightTheme = createLightTheme(phthaloGreenPalette);

// Export default theme
export default daemoniorumDarkTheme;
