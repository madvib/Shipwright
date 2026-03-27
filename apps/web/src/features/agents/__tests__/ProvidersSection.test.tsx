import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup } from '@testing-library/react'
import { ProvidersSection } from '../sections/ProvidersSection'

// Mock ProviderLogo to avoid image loading in tests
vi.mock('#/features/compiler/ProviderLogo', () => ({
  ProviderLogo: ({ provider }: { provider: string }) => <span data-testid={`logo-${provider}`}>{provider}</span>,
}))

// Mock ProviderConfigModal to avoid deep dependency chains
vi.mock('../dialogs/ProviderConfigModal', () => ({
  ProviderConfigModal: () => <div data-testid="config-modal" />,
}))

afterEach(cleanup)

const baseProps = {
  providers: ['claude', 'gemini', 'codex'],
  model: null as string | null,
  env: null as Record<string, string> | null,
  availableModels: null as string[] | null,
  agentLimits: null as { max_turns?: number; max_cost_per_session?: number } | null,
  hooks: [],
  providerSettings: {},
  onChangeModel: vi.fn(),
  onChangeEnv: vi.fn(),
  onChangeAvailableModels: vi.fn(),
  onChangeAgentLimits: vi.fn(),
  onChangeHooks: vi.fn(),
  onChangeProviderSettings: vi.fn(),
}

describe('ProvidersSection', () => {
  it('renders one card per provider', () => {
    render(<ProvidersSection {...baseProps} />)
    const buttons = screen.getAllByRole('button')
    expect(buttons).toHaveLength(3)
    expect(screen.getByText('Claude')).toBeTruthy()
    expect(screen.getByText('Gemini')).toBeTruthy()
    expect(screen.getByText('Codex')).toBeTruthy()
  })

  it('returns null when providers is empty', () => {
    const { container } = render(<ProvidersSection {...baseProps} providers={[]} />)
    expect(container.innerHTML).toBe('')
  })

  it('shows model in summary when set', () => {
    render(<ProvidersSection {...baseProps} model="claude-sonnet-4" />)
    expect(screen.getAllByText(/claude-sonnet-4/).length).toBeGreaterThan(0)
  })

  it('shows defaults when no customizations', () => {
    render(<ProvidersSection {...baseProps} />)
    const defaults = screen.getAllByText('defaults')
    expect(defaults.length).toBe(3)
  })

  it('shows hook count for claude and gemini only', () => {
    render(
      <ProvidersSection
        {...baseProps}
        hooks={[
          { id: '1', trigger: 'PreToolUse', command: 'echo test' },
          { id: '2', trigger: 'Stop', command: 'echo done' },
        ]}
      />,
    )
    // Claude and Gemini show hook counts, Codex shows defaults
    const hookTexts = screen.getAllByText(/2 hooks/)
    expect(hookTexts.length).toBe(2)
  })

  it('derives provider count from props, not hardcoded', () => {
    render(<ProvidersSection {...baseProps} providers={['cursor']} />)
    expect(screen.getByText('Cursor')).toBeTruthy()
    expect(screen.queryByText('Claude')).toBeNull()
  })

  it('shows provider settings count in summary', () => {
    render(
      <ProvidersSection
        {...baseProps}
        providers={['claude']}
        providerSettings={{ claude: { theme: 'dark', auto_updates: true, model: 'test' } }}
      />,
    )
    expect(screen.getByText(/3 custom settings/)).toBeTruthy()
  })

  it('renders correct count text in section header', () => {
    render(<ProvidersSection {...baseProps} />)
    expect(screen.getByText('3 providers')).toBeTruthy()
  })

  it('renders singular provider text', () => {
    render(<ProvidersSection {...baseProps} providers={['claude']} />)
    expect(screen.getByText('1 provider')).toBeTruthy()
  })
})
