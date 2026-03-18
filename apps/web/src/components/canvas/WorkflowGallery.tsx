import { PRESETS, type PresetInfo } from './presets'

/* ── Injected styles for gallery animations ── */

const GALLERY_STYLES = `
  @keyframes wf-card-in {
    from { opacity: 0; transform: translateY(12px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  @keyframes wf-glow-pulse {
    0%, 100% { opacity: 0.4; }
    50%      { opacity: 0.8; }
  }
  .wf-card {
    animation: wf-card-in 0.4s ease both;
    transition: border-color 0.2s, box-shadow 0.2s, transform 0.2s;
  }
  .wf-card:hover {
    transform: translateY(-2px);
  }
  .wf-card:hover .wf-use-btn {
    background: var(--wf-accent) !important;
    border-color: var(--wf-accent) !important;
    color: #000 !important;
  }
  .wf-card:hover .wf-preview-glow {
    opacity: 1;
  }
`

/* ── Mini SVG previews (static, per preset) ── */

function ShipflowPreview() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 230 170">
      <line x1="75" y1="38" x2="100" y2="65" stroke="#7c3aed44" strokeWidth="1.5" strokeDasharray="4,3"/>
      <line x1="105" y1="95" x2="68" y2="115" stroke="#f59e0b66" strokeWidth="1.5"/>
      <line x1="115" y1="95" x2="152" y2="115" stroke="#f59e0b66" strokeWidth="1.5"/>
      <line x1="65" y1="138" x2="52" y2="152" stroke="#22c55e44" strokeWidth="1" strokeDasharray="2,2"/>
      <line x1="153" y1="138" x2="165" y2="152" stroke="#22c55e44" strokeWidth="1" strokeDasharray="2,2"/>
      <circle cx="75" cy="28" r="14" fill="#7c3aed11" stroke="#7c3aed" strokeWidth="1.5"/>
      <text x="75" y="32" textAnchor="middle" fontSize="11">👤</text>
      <rect x="78" y="65" width="58" height="30" rx="5" fill="#f59e0b0d" stroke="#f59e0b" strokeWidth="1.2"/>
      <text x="107" y="77" textAnchor="middle" fontSize="7" fill="#f59e0b" fontWeight="700">commander</text>
      <text x="107" y="87" textAnchor="middle" fontSize="6" fill="#f59e0b66">planner · dispatch · gate</text>
      <rect x="38" y="115" width="52" height="23" rx="4" fill="#22c55e0a" stroke="#22c55e44" strokeWidth="1"/>
      <text x="64" y="130" textAnchor="middle" fontSize="7" fill="#22c55e88">web-lane</text>
      <rect x="118" y="115" width="58" height="23" rx="4" fill="#22c55e0a" stroke="#22c55e44" strokeWidth="1"/>
      <text x="147" y="130" textAnchor="middle" fontSize="7" fill="#22c55e88">rust-runtime</text>
      <path d="M186,28 h30 v28 l-6,6 H186 Z" fill="#7c3aed08" stroke="#7c3aed33" strokeWidth="1"/>
      <text x="201" y="44" textAnchor="middle" fontSize="6" fill="#7c3aed66">target</text>
    </svg>
  )
}

function SuperpowersPreview() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 230 170">
      <circle cx="110" cy="28" r="14" fill="#7c3aed11" stroke="#7c3aed" strokeWidth="1.5"/>
      <text x="110" y="32" textAnchor="middle" fontSize="11">👤</text>
      <line x1="110" y1="42" x2="75" y2="60" stroke="#7c3aed55" strokeWidth="1.5" strokeDasharray="4,3"/>
      <rect x="30" y="60" width="60" height="26" rx="5" fill="#7c3aed0d" stroke="#7c3aed" strokeWidth="1.2"/>
      <text x="60" y="70" textAnchor="middle" fontSize="7" fill="#7c3aed" fontWeight="700">brainstorm</text>
      <text x="60" y="79" textAnchor="middle" fontSize="6" fill="#7c3aed66">spec out</text>
      <rect x="145" y="60" width="56" height="26" rx="5" fill="#38bdf80d" stroke="#38bdf8" strokeWidth="1.2"/>
      <text x="173" y="70" textAnchor="middle" fontSize="7" fill="#38bdf8" fontWeight="700">write-plans</text>
      <rect x="30" y="115" width="60" height="26" rx="5" fill="#22c55e0d" stroke="#22c55e" strokeWidth="1.2"/>
      <text x="60" y="125" textAnchor="middle" fontSize="7" fill="#22c55e" fontWeight="700">execute</text>
      <rect x="145" y="115" width="56" height="26" rx="5" fill="#f59e0b0d" stroke="#f59e0b" strokeWidth="1.2"/>
      <text x="173" y="125" textAnchor="middle" fontSize="7" fill="#f59e0b" fontWeight="700">verify</text>
      <line x1="90" y1="73" x2="145" y2="73" stroke="#38bdf844" strokeWidth="1.5"/>
      <line x1="173" y1="86" x2="173" y2="115" stroke="#22c55e44" strokeWidth="1.5"/>
      <line x1="145" y1="128" x2="90" y2="128" stroke="#f59e0b44" strokeWidth="1.5"/>
      <path d="M201,128 C215,128 220,50 185,50 C170,50 165,60 165,60" fill="none" stroke="#f59e0b33" strokeWidth="1" strokeDasharray="3,3"/>
    </svg>
  )
}

function GstackPreview() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 230 170">
      <circle cx="22" cy="80" r="12" fill="#7c3aed11" stroke="#7c3aed" strokeWidth="1.5"/>
      <text x="22" y="84" textAnchor="middle" fontSize="9">👤</text>
      <rect x="42" y="68" width="36" height="24" rx="4" fill="#38bdf80a" stroke="#38bdf844" strokeWidth="1"/>
      <text x="60" y="78" textAnchor="middle" fontSize="6" fill="#38bdf8" fontWeight="700">spec</text>
      <rect x="90" y="68" width="36" height="24" rx="4" fill="#a78bfa0a" stroke="#a78bfa44" strokeWidth="1"/>
      <text x="108" y="78" textAnchor="middle" fontSize="6" fill="#a78bfa" fontWeight="700">design</text>
      <rect x="138" y="68" width="36" height="24" rx="4" fill="#22c55e0a" stroke="#22c55e44" strokeWidth="1"/>
      <text x="156" y="78" textAnchor="middle" fontSize="6" fill="#22c55e" fontWeight="700">impl</text>
      <rect x="186" y="68" width="36" height="24" rx="4" fill="#f59e0b0a" stroke="#f59e0b44" strokeWidth="1"/>
      <text x="204" y="78" textAnchor="middle" fontSize="6" fill="#f59e0b" fontWeight="700">review</text>
      <rect x="138" y="120" width="68" height="22" rx="4" fill="#f43f5e0a" stroke="#f43f5e33" strokeWidth="1"/>
      <text x="172" y="134" textAnchor="middle" fontSize="7" fill="#f43f5e88" fontWeight="700">deploy</text>
      <line x1="34" y1="80" x2="42" y2="80" stroke="#7c3aed44" strokeWidth="1.5" strokeDasharray="3,2"/>
      <line x1="78" y1="80" x2="90" y2="80" stroke="#38bdf844" strokeWidth="1.5"/>
      <line x1="126" y1="80" x2="138" y2="80" stroke="#a78bfa44" strokeWidth="1.5"/>
      <line x1="174" y1="80" x2="186" y2="80" stroke="#22c55e44" strokeWidth="1.5"/>
      <line x1="204" y1="92" x2="190" y2="120" stroke="#f59e0b44" strokeWidth="1.5"/>
      <path d="M186,74 C186,50 138,50 138,68" fill="none" stroke="#f43f5e33" strokeWidth="1" strokeDasharray="3,3"/>
    </svg>
  )
}

function ShipflowSoloPreview() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 230 170">
      <circle cx="110" cy="30" r="14" fill="#7c3aed11" stroke="#7c3aed" strokeWidth="1.5"/>
      <text x="110" y="34" textAnchor="middle" fontSize="11">👤</text>
      <line x1="110" y1="44" x2="110" y2="70" stroke="#7c3aed44" strokeWidth="1.5" strokeDasharray="4,3"/>
      <rect x="60" y="70" width="100" height="38" rx="6" fill="#f59e0b0d" stroke="#f59e0b" strokeWidth="1.5"/>
      <text x="110" y="85" textAnchor="middle" fontSize="8" fill="#f59e0b" fontWeight="700">commander</text>
      <text x="110" y="97" textAnchor="middle" fontSize="6.5" fill="#f59e0b66">solo · all skills loaded</text>
      <path d="M160,89 C185,89 185,130 145,135 C125,138 90,138 75,135 C55,130 55,89 60,89" fill="none" stroke="#f59e0b22" strokeWidth="1" strokeDasharray="3,3"/>
      <rect x="75" y="142" width="70" height="18" rx="3" fill="#111" stroke="#2a2a2a"/>
      <text x="110" y="154" textAnchor="middle" fontSize="7" fill="#555">session log</text>
      <line x1="110" y1="108" x2="110" y2="142" stroke="#22c55e44" strokeWidth="1" strokeDasharray="2,2"/>
    </svg>
  )
}

function SuperpowersSoloPreview() {
  return (
    <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" viewBox="0 0 230 170">
      <circle cx="110" cy="28" r="14" fill="#7c3aed11" stroke="#7c3aed" strokeWidth="1.5"/>
      <text x="110" y="32" textAnchor="middle" fontSize="11">👤</text>
      <line x1="110" y1="42" x2="110" y2="65" stroke="#7c3aed55" strokeWidth="1.5" strokeDasharray="4,3"/>
      <rect x="55" y="65" width="110" height="50" rx="6" fill="#7c3aed0d" stroke="#7c3aed" strokeWidth="1.5"/>
      <text x="110" y="82" textAnchor="middle" fontSize="8" fill="#7c3aed" fontWeight="700">superpowers agent</text>
      <rect x="65" y="87" width="38" height="12" rx="3" fill="#7c3aed22" stroke="#7c3aed33"/>
      <text x="84" y="96" textAnchor="middle" fontSize="6" fill="#7c3aed88">brainstorm</text>
      <rect x="107" y="87" width="30" height="12" rx="3" fill="#7c3aed22" stroke="#7c3aed33"/>
      <text x="122" y="96" textAnchor="middle" fontSize="6" fill="#7c3aed88">plan</text>
      <rect x="141" y="87" width="16" height="12" rx="3" fill="#7c3aed22" stroke="#7c3aed33"/>
      <text x="149" y="96" textAnchor="middle" fontSize="6" fill="#7c3aed88">...</text>
      <line x1="110" y1="115" x2="110" y2="138" stroke="#22c55e44" strokeWidth="1" strokeDasharray="2,2"/>
      <rect x="75" y="138" width="70" height="18" rx="3" fill="#111" stroke="#2a2a2a"/>
      <text x="110" y="150" textAnchor="middle" fontSize="7" fill="#555">session log</text>
    </svg>
  )
}

function BlankPreview() {
  return (
    <svg width="80" height="80" xmlns="http://www.w3.org/2000/svg">
      <rect x="10" y="30" width="26" height="20" rx="4" fill="#111" stroke="#2a2a2a" strokeWidth="1" strokeDasharray="3,2"/>
      <rect x="45" y="30" width="26" height="20" rx="4" fill="#111" stroke="#2a2a2a" strokeWidth="1" strokeDasharray="3,2"/>
      <line x1="36" y1="40" x2="45" y2="40" stroke="#2a2a2a" strokeWidth="1" strokeDasharray="3,2"/>
      <text x="40" y="68" textAnchor="middle" fontSize="8" fill="#333">start blank</text>
    </svg>
  )
}

const PREVIEW_MAP: Record<string, () => React.ReactNode> = {
  shipflow: () => <ShipflowPreview />,
  superpowers: () => <SuperpowersPreview />,
  gstack: () => <GstackPreview />,
  'shipflow-solo': () => <ShipflowSoloPreview />,
  'superpowers-solo': () => <SuperpowersSoloPreview />,
  blank: () => <BlankPreview />,
}

/* ── Card ── */

function PresetCard({ preset, onSelect, index }: { preset: PresetInfo; onSelect: (id: string) => void; index: number }) {
  const preview = PREVIEW_MAP[preset.id]

  return (
    <div
      className="wf-card"
      onClick={() => onSelect(preset.id)}
      style={{
        '--wf-accent': preset.accentColor,
        background: '#0d0d0d',
        border: '1px solid #1a1a1a',
        borderRadius: 12,
        overflow: 'hidden',
        cursor: 'pointer',
        animationDelay: `${index * 60}ms`,
      } as React.CSSProperties}
      onMouseEnter={(e) => {
        e.currentTarget.style.borderColor = preset.accentColor + '66'
        e.currentTarget.style.boxShadow = `0 4px 24px ${preset.accentColor}15, 0 0 0 1px ${preset.accentColor}22`
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.borderColor = '#1a1a1a'
        e.currentTarget.style.boxShadow = 'none'
      }}
    >
      {/* Graph preview */}
      <div style={{
        height: 160,
        background: '#060608',
        borderBottom: '1px solid #141418',
        position: 'relative',
        overflow: 'hidden',
        display: preset.id === 'blank' ? 'flex' : undefined,
        alignItems: preset.id === 'blank' ? 'center' : undefined,
        justifyContent: preset.id === 'blank' ? 'center' : undefined,
      }}>
        {/* Subtle radial glow behind the preview on hover */}
        <div
          className="wf-preview-glow"
          style={{
            position: 'absolute', inset: 0, opacity: 0, transition: 'opacity 0.3s',
            background: `radial-gradient(ellipse at 50% 60%, ${preset.accentColor}08 0%, transparent 70%)`,
          }}
        />
        {preview?.()}
      </div>

      {/* Card body */}
      <div style={{ padding: '14px 16px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
          <span style={{
            fontFamily: 'var(--font-display-app)',
            fontSize: 14,
            fontWeight: 700,
            color: '#f0f0f0',
            letterSpacing: '-0.01em',
          }}>
            {preset.name}
          </span>
          {preset.badge && (
            <span style={{
              fontFamily: 'var(--font-body-app)',
              fontSize: 8,
              fontWeight: 700,
              letterSpacing: '0.08em',
              padding: '2px 7px',
              borderRadius: 3,
              textTransform: 'uppercase',
              ...(preset.badge === 'ship'
                ? { background: '#7c3aed18', color: '#a78bfa', border: '1px solid #7c3aed28' }
                : { background: '#38bdf818', color: '#7dd3fc', border: '1px solid #38bdf828' }),
            }}>
              {preset.badge === 'ship' ? 'SHIP' : 'COMMUNITY'}
            </span>
          )}
        </div>

        <p style={{
          fontFamily: 'var(--font-body-app)',
          fontSize: 11,
          color: '#555',
          lineHeight: 1.6,
          marginBottom: 12,
        }}>
          {preset.description}
        </p>

        {/* Agents row */}
        <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap', marginBottom: 12 }}>
          {preset.agents.length > 0 ? preset.agents.map((a) => (
            <div key={a.name} style={{
              display: 'flex', alignItems: 'center', gap: 5,
              background: '#0a0a0e', border: '1px solid #1a1a1e', borderRadius: 4,
              padding: '3px 8px',
              fontFamily: 'var(--font-body-app)',
              fontSize: 9, color: '#666',
            }}>
              <span style={{
                width: 5, height: 5, borderRadius: '50%', background: a.color,
                boxShadow: `0 0 4px ${a.color}66`,
              }} />
              {a.name}
            </div>
          )) : (
            <div style={{
              display: 'flex', alignItems: 'center', gap: 4,
              background: '#0a0a0e', border: '1px dashed #1a1a1e', borderRadius: 4,
              padding: '3px 8px',
              fontFamily: 'var(--font-body-app)',
              fontSize: 9, color: '#3a3a3a',
            }}>
              + you design it
            </div>
          )}
        </div>

        {/* Use button */}
        <div
          className="wf-use-btn"
          style={{
            width: '100%',
            padding: 8,
            background: '#111114',
            border: '1px solid #1e1e24',
            borderRadius: 6,
            fontFamily: 'var(--font-body-app)',
            fontSize: 11,
            fontWeight: 600,
            color: '#666',
            textAlign: 'center',
            transition: 'all 0.15s',
            letterSpacing: '0.01em',
          }}
        >
          {preset.id === 'blank' ? 'Start blank' : `Use ${preset.name}`}
        </div>
      </div>
    </div>
  )
}

/* ── Gallery ── */

interface Props {
  onSelect: (presetId: string) => void
}

export function WorkflowGallery({ onSelect }: Props) {
  return (
    <>
      <style>{GALLERY_STYLES}</style>
      <div style={{
        background: '#08080a',
        minHeight: '100%',
        padding: '32px 32px 48px',
        overflowY: 'auto',
      }}>
        {/* Subtle top gradient */}
        <div style={{
          position: 'fixed', top: 53, left: 0, right: 0, height: 120,
          background: 'linear-gradient(to bottom, #08080a 0%, transparent 100%)',
          pointerEvents: 'none', zIndex: 1,
        }} />

        <div style={{ position: 'relative', zIndex: 2 }}>
          <h2 style={{
            fontFamily: 'var(--font-display-app)',
            fontSize: 22,
            fontWeight: 800,
            color: '#f0f0f0',
            marginBottom: 6,
            letterSpacing: '-0.02em',
          }}>
            Choose a workflow
          </h2>
          <p style={{
            fontFamily: 'var(--font-body-app)',
            fontSize: 12,
            color: '#444',
            marginBottom: 28,
            lineHeight: 1.6,
          }}>
            Shipped as presets. Activate via CLI:{' '}
            <code style={{
              fontFamily: 'ui-monospace, "Fira Code", monospace',
              color: '#a78bfa',
              background: '#7c3aed0c',
              padding: '2px 7px',
              borderRadius: 4,
              fontSize: 11,
              border: '1px solid #7c3aed15',
            }}>
              ship use workflow/shipflow
            </code>
          </p>
          <div style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(3, 1fr)',
            gap: 16,
            maxWidth: 980,
          }}>
            {PRESETS.map((preset, i) => (
              <PresetCard key={preset.id} preset={preset} onSelect={onSelect} index={i} />
            ))}
          </div>
        </div>
      </div>
    </>
  )
}
