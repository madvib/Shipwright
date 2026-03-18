// Tech icon tiles using Simple Icons CDN for real brand SVGs.
// https://cdn.simpleicons.org/{slug}/{hex-color}

export const TECH_STACKS = {
  react:      { slug: 'react',          fg: '#61dafb', bg: '#20232a', border: '#61dafb33' },
  typescript: { slug: 'typescript',     fg: '#fff',    bg: '#3178c6', border: '#3178c666' },
  javascript: { slug: 'javascript',     fg: '#000',    bg: '#f7df1e', border: '#f7df1e66' },
  rust:       { slug: 'rust',           fg: '#ce422b', bg: '#1a0a08', border: '#ce422b33' },
  go:         { slug: 'go',             fg: '#00acd7', bg: '#00acd722', border: '#00acd733' },
  python:     { slug: 'python',         fg: '#fff',    bg: '#3776ab', border: '#3776ab66' },
  git:        { slug: 'git',            fg: '#fff',    bg: '#f05028', border: '#f0502866' },
  nextjs:     { slug: 'nextdotjs',      fg: '#fff',    bg: '#000',    border: '#ffffff22' },
  tailwind:   { slug: 'tailwindcss',    fg: '#fff',    bg: '#06b6d4', border: '#06b6d466' },
  docker:     { slug: 'docker',         fg: '#fff',    bg: '#2496ed', border: '#2496ed66' },
  cloudflare: { slug: 'cloudflare',     fg: '#fff',    bg: '#f6821f', border: '#f6821f66' },
  postgres:   { slug: 'postgresql',     fg: '#fff',    bg: '#336791', border: '#33679166' },
  node:       { slug: 'nodedotjs',      fg: '#fff',    bg: '#339933', border: '#33993366' },
  vue:        { slug: 'vuedotjs',       fg: '#fff',    bg: '#42b883', border: '#42b88366' },
  svelte:     { slug: 'svelte',         fg: '#fff',    bg: '#ff3e00', border: '#ff3e0066' },
  custom:     { slug: null,             fg: '#000',    bg: '#f59e0b', border: '#f59e0b66' },
} as const

export type TechStack = keyof typeof TECH_STACKS

export const TECH_STACK_LIST = Object.entries(TECH_STACKS).map(([id, v]) => ({ id: id as TechStack, ...v }))

interface TechIconProps {
  stack: string
  size?: number
  className?: string
  style?: React.CSSProperties
}

export function TechIcon({ stack, size = 36, className = '', style }: TechIconProps) {
  const tech = TECH_STACKS[stack as TechStack] ?? TECH_STACKS.custom
  const iconSize = Math.round(size * 0.55)
  const fg = tech.fg.replace('#', '')

  return (
    <div
      className={`shrink-0 flex items-center justify-center ${className}`}
      style={{
        width: size,
        height: size,
        background: tech.bg,
        border: `1px solid ${tech.border}`,
        borderRadius: 8,
        ...style,
      }}
    >
      {tech.slug ? (
        <img
          src={`https://cdn.simpleicons.org/${tech.slug}/${fg}`}
          alt={stack}
          width={iconSize}
          height={iconSize}
          style={{ display: 'block' }}
        />
      ) : (
        <span
          style={{
            fontSize: size <= 24 ? 8 : size <= 32 ? 10 : 12,
            fontFamily: 'monospace',
            fontWeight: 700,
            color: tech.fg,
            letterSpacing: '-0.03em',
          }}
        >
          {stack.slice(0, 2).toUpperCase()}
        </span>
      )}
    </div>
  )
}
