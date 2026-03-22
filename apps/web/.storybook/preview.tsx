import '../src/styles.css'
import React from 'react'
import type { Preview } from '@storybook/react'

/**
 * Wraps each story in a container that applies the project's dark class
 * based on the selected Storybook background, so Tailwind dark: variants
 * and CSS custom properties resolve correctly.
 */
const preview: Preview = {
  parameters: {
    backgrounds: {
      default: 'dark',
      values: [
        { name: 'dark', value: '#1a1a2e' },
        { name: 'light', value: '#fafaf9' },
      ],
    },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
  decorators: [
    (Story, context) => {
      const bgName = context.globals?.backgrounds?.value
      const isDark = bgName !== '#fafaf9'

      return (
        <div
          className={isDark ? 'dark' : ''}
          style={{
            minHeight: '100%',
            padding: '1.5rem',
            fontFamily: 'var(--font-sans)',
            background: isDark ? '#1a1a2e' : '#fafaf9',
            color: isDark ? '#f5f5f4' : '#1c1917',
          }}
        >
          <Story />
        </div>
      )
    },
  ],
}

export default preview
