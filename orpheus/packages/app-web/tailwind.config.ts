import type { Config } from 'tailwindcss'
import baseConfig from '@persona-framework/ui/tailwind.config'

export default {
  presets: [baseConfig],
  content: [
    './index.html',
    './src/**/*.{js,ts,jsx,tsx}',
    '../../packages/persona-ui/src/**/*.{js,ts,jsx,tsx}',
  ],
} satisfies Config
