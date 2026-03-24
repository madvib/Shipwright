import { createContext, useContext } from 'react'
import { useLocalMcp } from './useLocalMcp'
import type { UseLocalMcpReturn } from './useLocalMcp'

const LocalMcpContext = createContext<UseLocalMcpReturn | null>(null)

export function LocalMcpProvider({ children }: { children: React.ReactNode }) {
  const mcp = useLocalMcp()
  return <LocalMcpContext.Provider value={mcp}>{children}</LocalMcpContext.Provider>
}

export function useLocalMcpContext(): UseLocalMcpReturn | null {
  return useContext(LocalMcpContext)
}
