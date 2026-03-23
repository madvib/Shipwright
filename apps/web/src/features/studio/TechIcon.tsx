// Tech icon tiles using Simple Icons CDN for real brand SVGs.
// https://cdn.simpleicons.org/{slug}/{hex-color}

export const TECH_STACKS = {
  // ── Languages ──────────────────────────────────────────────────────────
  typescript: { slug: 'typescript',     fg: '#fff',    bg: '#3178c6', border: '#3178c666' },
  javascript: { slug: 'javascript',     fg: '#000',    bg: '#f7df1e', border: '#f7df1e66' },
  python:     { slug: 'python',         fg: '#fff',    bg: '#3776ab', border: '#3776ab66' },
  rust:       { slug: 'rust',           fg: '#ce422b', bg: '#1a0a08', border: '#ce422b33' },
  go:         { slug: 'go',             fg: '#00acd7', bg: '#00acd722', border: '#00acd733' },
  java:       { slug: 'openjdk',        fg: '#fff',    bg: '#ed8b00', border: '#ed8b0066' },
  csharp:     { slug: 'csharp',         fg: '#fff',    bg: '#512bd4', border: '#512bd466' },
  swift:      { slug: 'swift',          fg: '#fff',    bg: '#f05138', border: '#f0513866' },
  kotlin:     { slug: 'kotlin',         fg: '#fff',    bg: '#7f52ff', border: '#7f52ff66' },
  ruby:       { slug: 'ruby',           fg: '#fff',    bg: '#cc342d', border: '#cc342d66' },
  php:        { slug: 'php',            fg: '#fff',    bg: '#777bb4', border: '#777bb466' },
  elixir:     { slug: 'elixir',         fg: '#fff',    bg: '#4b275f', border: '#4b275f66' },

  // ── Frameworks ─────────────────────────────────────────────────────────
  react:      { slug: 'react',          fg: '#61dafb', bg: '#20232a', border: '#61dafb33' },
  nextjs:     { slug: 'nextdotjs',      fg: '#fff',    bg: '#000',    border: '#ffffff22' },
  vue:        { slug: 'vuedotjs',       fg: '#fff',    bg: '#42b883', border: '#42b88366' },
  svelte:     { slug: 'svelte',         fg: '#fff',    bg: '#ff3e00', border: '#ff3e0066' },
  angular:    { slug: 'angular',        fg: '#fff',    bg: '#dd0031', border: '#dd003166' },
  astro:      { slug: 'astro',          fg: '#fff',    bg: '#bc52ee', border: '#bc52ee66' },
  tailwind:   { slug: 'tailwindcss',    fg: '#fff',    bg: '#06b6d4', border: '#06b6d466' },
  django:     { slug: 'django',         fg: '#fff',    bg: '#092e20', border: '#09434066' },
  rails:      { slug: 'rubyonrails',    fg: '#fff',    bg: '#d30001', border: '#d3000166' },
  flask:      { slug: 'flask',          fg: '#fff',    bg: '#000',    border: '#ffffff22' },
  fastapi:    { slug: 'fastapi',        fg: '#fff',    bg: '#009688', border: '#00968866' },

  // ── Infrastructure ─────────────────────────────────────────────────────
  docker:     { slug: 'docker',         fg: '#fff',    bg: '#2496ed', border: '#2496ed66' },
  kubernetes: { slug: 'kubernetes',     fg: '#fff',    bg: '#326ce5', border: '#326ce566' },
  terraform:  { slug: 'terraform',      fg: '#fff',    bg: '#844fba', border: '#844fba66' },
  aws:        { slug: 'amazonaws',      fg: '#fff',    bg: '#232f3e', border: '#ff990066' },
  gcp:        { slug: 'googlecloud',    fg: '#fff',    bg: '#4285f4', border: '#4285f466' },
  azure:      { slug: 'microsoftazure', fg: '#fff',    bg: '#0078d4', border: '#0078d466' },
  cloudflare: { slug: 'cloudflare',     fg: '#fff',    bg: '#f6821f', border: '#f6821f66' },
  vercel:     { slug: 'vercel',         fg: '#fff',    bg: '#000',    border: '#ffffff22' },
  nginx:      { slug: 'nginx',          fg: '#fff',    bg: '#009639', border: '#00963966' },

  // ── Data ───────────────────────────────────────────────────────────────
  postgres:   { slug: 'postgresql',     fg: '#fff',    bg: '#336791', border: '#33679166' },
  mysql:      { slug: 'mysql',          fg: '#fff',    bg: '#4479a1', border: '#4479a166' },
  mongodb:    { slug: 'mongodb',        fg: '#fff',    bg: '#47a248', border: '#47a24866' },
  redis:      { slug: 'redis',          fg: '#fff',    bg: '#dc382d', border: '#dc382d66' },
  sqlite:     { slug: 'sqlite',         fg: '#fff',    bg: '#003b57', border: '#003b5766' },
  graphql:    { slug: 'graphql',        fg: '#fff',    bg: '#e10098', border: '#e1009866' },

  // ── Tools ──────────────────────────────────────────────────────────────
  git:        { slug: 'git',            fg: '#fff',    bg: '#f05028', border: '#f0502866' },
  github:     { slug: 'github',         fg: '#fff',    bg: '#181717', border: '#ffffff22' },
  node:       { slug: 'nodedotjs',      fg: '#fff',    bg: '#339933', border: '#33993366' },
  bun:        { slug: 'bun',            fg: '#fff',    bg: '#000',    border: '#fbf0df44' },
  deno:       { slug: 'deno',           fg: '#fff',    bg: '#000',    border: '#ffffff22' },
  linux:      { slug: 'linux',          fg: '#fff',    bg: '#fcc624', border: '#fcc62466' },
  vim:        { slug: 'vim',            fg: '#fff',    bg: '#019733', border: '#01973366' },

  // ── AI / ML ────────────────────────────────────────────────────────────
  openai:     { slug: 'openai',         fg: '#fff',    bg: '#412991', border: '#41299166' },
  anthropic:  { slug: 'anthropic',      fg: '#fff',    bg: '#191919', border: '#d4a27444' },
  huggingface:{ slug: 'huggingface',    fg: '#000',    bg: '#ffd21e', border: '#ffd21e66' },
  pytorch:    { slug: 'pytorch',        fg: '#fff',    bg: '#ee4c2c', border: '#ee4c2c66' },

  // ── Generic roles (no brand icon — uses initials) ──────────────────────
  code:       { slug: 'codecov',        fg: '#fff',    bg: '#f01f7a', border: '#f01f7a66' },
  security:   { slug: 'letsencrypt',    fg: '#fff',    bg: '#003a70', border: '#003a7066' },
  test:       { slug: 'testinglibrary', fg: '#fff',    bg: '#e33332', border: '#e3333266' },
  docs:       { slug: 'readthedocs',    fg: '#fff',    bg: '#8ca1af', border: '#8ca1af66' },
  api:        { slug: 'swagger',        fg: '#fff',    bg: '#85ea2d', border: '#85ea2d66' },
  deploy:     { slug: 'githubactions',  fg: '#fff',    bg: '#2088ff', border: '#2088ff66' },
  monitor:    { slug: 'grafana',        fg: '#fff',    bg: '#f46800', border: '#f4680066' },
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
