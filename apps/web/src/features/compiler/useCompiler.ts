import { useState, useCallback, useRef } from 'react'
import { toast } from 'sonner'
import type { ProjectLibrary } from '@ship/ui'
import type { CompileResult } from './types'

type WasmModule = {
  compileLibraryAll: (library_json: string, active_mode?: string | null) => string
  listProviders: () => string[]
}

let wasmModule: WasmModule | null = null
let wasmLoading: Promise<WasmModule> | null = null

async function loadWasm(): Promise<WasmModule> {
  if (wasmModule) return wasmModule
  if (wasmLoading) return wasmLoading
  wasmLoading = (async () => {
    const mod = await import('@ship/compiler')
    await mod.default()
    wasmModule = mod as unknown as WasmModule
    return wasmModule
  })()
  return wasmLoading
}

export type CompileState =
  | { status: 'idle' }
  | { status: 'compiling' }
  | { status: 'ok'; output: Record<string, CompileResult>; elapsed: number }
  | { status: 'error'; message: string }

export function useCompiler() {
  const [state, setState] = useState<CompileState>({ status: 'idle' })
  const abortRef = useRef<AbortController | null>(null)

  const compile = useCallback(async (library: ProjectLibrary) => {
    abortRef.current?.abort()
    const ctrl = new AbortController()
    abortRef.current = ctrl

    setState({ status: 'compiling' })
    const t0 = performance.now()

    try {
      const wasm = await loadWasm()
      if (ctrl.signal.aborted) return

      const json = JSON.stringify(library)
      const raw = wasm.compileLibraryAll(json, library.active_agent ?? null)
      if (ctrl.signal.aborted) return

      const output = JSON.parse(raw) as Record<string, CompileResult>
      const elapsed = Math.round(performance.now() - t0)
      setState({ status: 'ok', output, elapsed })
    } catch (e) {
      if (ctrl.signal.aborted) return
      const message = e instanceof Error ? e.message : String(e)
      setState({ status: 'error', message })
      toast.error('Compile failed', { description: message })
    }
  }, [])

  const reset = useCallback(() => {
    abortRef.current?.abort()
    setState({ status: 'idle' })
  }, [])

  return { state, compile, reset }
}
