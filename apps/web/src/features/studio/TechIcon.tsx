// Tech icon tiles using Simple Icons CDN for real brand SVGs.
// https://cdn.simpleicons.org/{slug}/{hex-color}

import { useState } from 'react'

export const TECH_STACKS = {
  // ── Languages ──────────────────────────────────────────────────────────
  typescript: { slug: 'typescript',     fg: '#fff',    bg: '#3178c6', border: '#3178c666' },
  javascript: { slug: 'javascript',     fg: '#000',    bg: '#f7df1e', border: '#f7df1e66' },
  python:     { slug: 'python',         fg: '#fff',    bg: '#3776ab', border: '#3776ab66' },
  rust:       { slug: 'rust',           fg: '#ce422b', bg: '#1a0a08', border: '#ce422b33' },
  go:         { slug: 'go',             fg: '#00acd7', bg: '#00acd722', border: '#00acd733' },
  java:       { slug: 'openjdk',        fg: '#fff',    bg: '#ed8b00', border: '#ed8b0066' },
  csharp:     { slug: 'csharp',          fg: '#fff',    bg: '#512bd4', border: '#512bd466' },
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
  aws:        { slug: 'amazonwebservices', fg: '#fff', bg: '#232f3e', border: '#ff990066' },
  gcp:        { slug: 'googlecloud',    fg: '#fff',    bg: '#4285f4', border: '#4285f466' },
  azure:      { slug: 'microsoftazure', fg: '#fff',    bg: '#0089d6', border: '#0089d666' },
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
  linux:      { slug: 'linux',          fg: '#000',    bg: '#e5e5e5', border: '#00000022' },

  // ── AI / ML ────────────────────────────────────────────────────────────
  openai:     { slug: null,             fg: '#10a37f', bg: '#10a37f22', border: '#10a37f44' },
  anthropic:  { slug: 'anthropic',      fg: '#fff',    bg: '#191919', border: '#d4a27444' },
  pytorch:    { slug: 'pytorch',        fg: '#fff',    bg: '#ee4c2c', border: '#ee4c2c66' },
  tensorflow: { slug: 'tensorflow',     fg: '#fff',    bg: '#ff6f00', border: '#ff6f0066' },

  // ── Generic roles — use null slug, rendered with initials ───────────────
  orchestrator: { slug: null, fg: '#c084fc', bg: '#7c3aed22', border: '#7c3aed44' },
  reviewer:     { slug: null, fg: '#60a5fa', bg: '#3b82f622', border: '#3b82f644' },
  frontend:     { slug: null, fg: '#34d399', bg: '#10b98122', border: '#10b98144' },
  backend:      { slug: null, fg: '#fbbf24', bg: '#f59e0b22', border: '#f59e0b44' },
  devops:       { slug: null, fg: '#f87171', bg: '#ef444422', border: '#ef444444' },
  security:     { slug: null, fg: '#a78bfa', bg: '#8b5cf622', border: '#8b5cf644' },
  testing:      { slug: null, fg: '#fb923c', bg: '#f9731622', border: '#f9731644' },
  docs:         { slug: null, fg: '#94a3b8', bg: '#64748b22', border: '#64748b44' },
  data:         { slug: null, fg: '#2dd4bf', bg: '#14b8a622', border: '#14b8a644' },
  mobile:       { slug: null, fg: '#f472b6', bg: '#ec489922', border: '#ec489944' },
  fullstack:    { slug: null, fg: '#e2e8f0', bg: '#47556922', border: '#47556944' },
  custom:       { slug: null, fg: '#fbbf24', bg: '#f59e0b22', border: '#f59e0b44' },
} as const

export type TechStack = keyof typeof TECH_STACKS

export const TECH_STACK_LIST = Object.entries(TECH_STACKS).map(([id, v]) => ({ id: id as TechStack, ...v }))

export const ICON_CATEGORIES = [
  { id: 'languages', label: 'Languages', keys: ['typescript','javascript','python','rust','go','java','csharp','swift','kotlin','ruby','php','elixir'] },
  { id: 'frameworks', label: 'Frameworks', keys: ['react','nextjs','vue','svelte','angular','astro','tailwind','django','rails','flask','fastapi'] },
  { id: 'infra', label: 'Infra', keys: ['docker','kubernetes','terraform','aws','gcp','azure','cloudflare','vercel','nginx'] },
  { id: 'data', label: 'Data', keys: ['postgres','mysql','mongodb','redis','sqlite','graphql'] },
  { id: 'tools', label: 'Tools', keys: ['git','github','node','bun','deno','linux'] },
  { id: 'ai', label: 'AI', keys: ['openai','anthropic','pytorch','tensorflow'] },
  { id: 'roles', label: 'Roles', keys: ['orchestrator','reviewer','frontend','backend','devops','security','testing','docs','data','mobile','fullstack'] },
] as const

interface TechIconProps {
  stack: string
  size?: number
  className?: string
  style?: React.CSSProperties
}

export function TechIcon({ stack, size = 36, className = '', style }: TechIconProps) {
  const [imgFailed, setImgFailed] = useState(false)
  const tech = TECH_STACKS[stack as TechStack] ?? TECH_STACKS.custom
  const iconSize = Math.round(size * 0.55)
  const fg = tech.fg.replace('#', '')
  const showImg = tech.slug && !imgFailed

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
      {showImg ? (
        <img
          src={`https://cdn.simpleicons.org/${tech.slug}/${fg}`}
          alt=""
          width={iconSize}
          height={iconSize}
          style={{ display: 'block' }}
          onError={() => setImgFailed(true)}
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
