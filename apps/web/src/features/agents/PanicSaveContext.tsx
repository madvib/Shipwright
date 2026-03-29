import { createContext, useCallback, useContext, useRef, type ReactNode } from 'react'

interface PanicSaveContextValue {
  register: (fn: () => void) => () => void
  saveAll: () => void
}

const PanicSaveContext = createContext<PanicSaveContextValue>({
  register: () => () => {},
  saveAll: () => {},
})

export function PanicSaveProvider({ children }: { children: ReactNode }) {
  const fnsRef = useRef<Set<() => void>>(new Set())

  const register = useCallback((fn: () => void) => {
    fnsRef.current.add(fn)
    return () => { fnsRef.current.delete(fn) }
  }, [])

  const saveAll = useCallback(() => {
    for (const fn of fnsRef.current) fn()
  }, [])

  return (
    <PanicSaveContext.Provider value={{ register, saveAll }}>
      {children}
    </PanicSaveContext.Provider>
  )
}

export function usePanicSave() {
  return useContext(PanicSaveContext)
}
