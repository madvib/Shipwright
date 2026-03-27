import { describe, it, expect, vi, afterEach } from 'vitest'
import { render, screen, cleanup, fireEvent } from '@testing-library/react'
import { ProviderConfigModal } from '../dialogs/ProviderConfigModal'

// Mock sub-tabs to avoid deep dependency chains (Input, @base-ui, etc.)
vi.mock('../dialogs/ProviderGeneralTab', () => ({
  ProviderGeneralTab: () => (
    <div data-testid="general-tab">
      <span>Model override</span>
      <span>Available models</span>
      <span>Agent limits</span>
      <span>Max turns</span>
      <span>Max cost per session</span>
      <span>Environment variables</span>
    </div>
  ),
}))

vi.mock('../dialogs/ProviderHooksTab', () => ({
  ProviderHooksTab: () => <div data-testid="hooks-tab">Hooks editor</div>,
}))

vi.mock('../dialogs/ProviderPassthroughTab', () => ({
  ProviderPassthroughTab: () => (
    <div data-testid="passthrough-tab">
      <span>Provider-specific settings</span>
    </div>
  ),
}))

// Mock ProviderLogo to avoid image loading
vi.mock('#/features/compiler/ProviderLogo', () => ({
  ProviderLogo: ({ provider }: { provider: string }) => <span data-testid="provider-logo">{provider}</span>,
}))

afterEach(cleanup)

const baseProps = {
  open: true,
  onOpenChange: vi.fn(),
  provider: 'claude',
  model: null as string | null,
  env: {} as Record<string, string>,
  availableModels: [] as string[],
  agentLimits: {} as { max_turns?: number; max_cost_per_session?: number },
  hooks: [],
  providerSettings: {} as Record<string, unknown>,
  onSave: vi.fn(),
}

describe('ProviderConfigModal', () => {
  it('renders provider name in header', () => {
    render(<ProviderConfigModal {...baseProps} />)
    expect(screen.getByText('Claude Configuration')).toBeTruthy()
  })

  it('renders nothing when closed', () => {
    const { container } = render(<ProviderConfigModal {...baseProps} open={false} />)
    expect(container.innerHTML).toBe('')
  })

  it('shows General tab content by default', () => {
    render(<ProviderConfigModal {...baseProps} />)
    expect(screen.getByTestId('general-tab')).toBeTruthy()
    expect(screen.getByText('Model override')).toBeTruthy()
    expect(screen.getByText('Available models')).toBeTruthy()
    expect(screen.getByText('Agent limits')).toBeTruthy()
    expect(screen.getByText('Max turns')).toBeTruthy()
    expect(screen.getByText('Max cost per session')).toBeTruthy()
    expect(screen.getByText('Environment variables')).toBeTruthy()
  })

  it('shows Hooks tab for claude provider', () => {
    render(<ProviderConfigModal {...baseProps} provider="claude" />)
    const hooksTab = screen.getByText('Hooks')
    expect(hooksTab).toBeTruthy()
  })

  it('does not show Hooks tab for codex provider', () => {
    render(<ProviderConfigModal {...baseProps} provider="codex" />)
    expect(screen.queryByText('Hooks')).toBeNull()
  })

  it('shows Hooks tab for gemini provider', () => {
    render(<ProviderConfigModal {...baseProps} provider="gemini" />)
    expect(screen.getByText('Hooks')).toBeTruthy()
  })

  it('shows Passthrough tab button', () => {
    render(<ProviderConfigModal {...baseProps} />)
    expect(screen.getByText('Passthrough')).toBeTruthy()
  })

  it('switches to Passthrough tab on click', () => {
    render(<ProviderConfigModal {...baseProps} />)
    fireEvent.click(screen.getByText('Passthrough'))
    expect(screen.getByTestId('passthrough-tab')).toBeTruthy()
    expect(screen.getByText('Provider-specific settings')).toBeTruthy()
  })

  it('switches to Hooks tab on click', () => {
    render(<ProviderConfigModal {...baseProps} provider="claude" />)
    fireEvent.click(screen.getByText('Hooks'))
    expect(screen.getByTestId('hooks-tab')).toBeTruthy()
  })

  it('calls onSave with config when Save is clicked', () => {
    const onSave = vi.fn()
    render(<ProviderConfigModal {...baseProps} onSave={onSave} />)
    fireEvent.click(screen.getByText('Save'))
    expect(onSave).toHaveBeenCalledTimes(1)
    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        model: null,
        env: {},
        availableModels: [],
        hooks: [],
        providerSettings: {},
      }),
    )
  })

  it('calls onOpenChange(false) when Cancel is clicked', () => {
    const onOpenChange = vi.fn()
    render(<ProviderConfigModal {...baseProps} onOpenChange={onOpenChange} />)
    fireEvent.click(screen.getByText('Cancel'))
    expect(onOpenChange).toHaveBeenCalledWith(false)
  })

  it('does not show Hooks tab for cursor or opencode', () => {
    const { unmount } = render(<ProviderConfigModal {...baseProps} provider="cursor" />)
    expect(screen.queryByText('Hooks')).toBeNull()
    unmount()

    render(<ProviderConfigModal {...baseProps} provider="opencode" />)
    expect(screen.queryByText('Hooks')).toBeNull()
  })
})
