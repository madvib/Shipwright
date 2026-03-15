import {
  HeadContent,
  Scripts,
  createRootRouteWithContext,
  useRouterState,
} from '@tanstack/react-router'
import Footer from '../components/Footer'
import Header from '../components/Header'

import TanStackQueryProvider from '../integrations/tanstack-query/root-provider'

import appCss from '../styles.css?url'

import type { QueryClient } from '@tanstack/react-query'

interface MyRouterContext {
  queryClient: QueryClient
}

const THEME_INIT_SCRIPT = `(function(){try{var stored=window.localStorage.getItem('theme');var mode=(stored==='light'||stored==='dark'||stored==='auto')?stored:'auto';var prefersDark=window.matchMedia('(prefers-color-scheme: dark)').matches;var resolved=mode==='auto'?(prefersDark?'dark':'light'):mode;var root=document.documentElement;root.classList.remove('light','dark');root.classList.add(resolved);if(mode==='auto'){root.removeAttribute('data-theme')}else{root.setAttribute('data-theme',mode)}root.style.colorScheme=resolved;}catch(e){}})();`

export const Route = createRootRouteWithContext<MyRouterContext>()({
  head: () => ({
    meta: [
      { charSet: 'utf-8' },
      { name: 'viewport', content: 'width=device-width, initial-scale=1' },
      { title: 'Ship Studio — Configure your AI agents' },
      { name: 'description', content: 'Build once, export to Claude Code, Gemini CLI, Codex, and Cursor. MCP servers, skills, and permissions — all in sync.' },
      { property: 'og:title', content: 'Ship Studio' },
      { property: 'og:description', content: 'Build once, export to Claude Code, Gemini CLI, Codex, and Cursor.' },
      { property: 'og:url', content: 'https://getship.dev' },
    ],
    links: [
      { rel: 'stylesheet', href: appCss },
      { rel: 'icon', href: '/ship-logos/ship_logo.svg', type: 'image/svg+xml' },
      { rel: 'icon', href: '/favicon.ico', sizes: 'any' },
    ],
  }),
  shellComponent: RootDocument,
})

function RootDocument({ children }: { children: React.ReactNode }) {
  const isStudio = useRouterState({ select: (s) => s.location.pathname === '/studio' })

  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: THEME_INIT_SCRIPT }} />
        <HeadContent />
      </head>
      <body className={`font-sans antialiased [overflow-wrap:anywhere]${isStudio ? ' flex flex-col h-screen overflow-hidden' : ''}`}>
        <TanStackQueryProvider>
          <Header />
          {children}
          {!isStudio && <Footer />}
        </TanStackQueryProvider>
        <Scripts />
      </body>
    </html>
  )
}
