import type { WorkflowPreset } from '../types'
import { shipflow } from './shipflow'
import { superpowers } from './superpowers'
import { gstack } from './gstack'
import { shipflowSolo } from './shipflow-solo'
import { superpowersSolo } from './superpowers-solo'
import { blank } from './blank'

export interface PresetInfo {
  id: string
  name: string
  description: string
  badge: 'ship' | 'community' | null
  accentColor: string
  agents: { name: string; color: string }[]
  load: () => WorkflowPreset
}

export const PRESETS: PresetInfo[] = [
  {
    id: 'shipflow',
    name: 'Shipflow',
    description: 'Full Ship orchestration — commander dispatches specialists via worktrees, gate reviews before merge.',
    badge: 'ship',
    accentColor: '#f59e0b',
    agents: [
      { name: 'Commander', color: '#f59e0b' },
      { name: 'Planner', color: '#3b82f6' },
      { name: 'Web Agent', color: '#3b82f6' },
      { name: 'Rust Agent', color: '#3b82f6' },
      { name: 'Gate', color: '#10b981' },
    ],
    load: () => shipflow,
  },
  {
    id: 'superpowers',
    name: 'Superpowers',
    description: 'Brainstorm → Plan → Execute → Verify pipeline with reject loop and dual specialist workers.',
    badge: 'community',
    accentColor: '#7c3aed',
    agents: [
      { name: 'Brainstormer', color: '#7c3aed' },
      { name: 'Planner', color: '#3b82f6' },
      { name: 'Executor', color: '#f59e0b' },
      { name: 'Verifier', color: '#10b981' },
    ],
    load: () => superpowers,
  },
  {
    id: 'gstack',
    name: 'G-Stack Pipeline',
    description: 'Linear pipeline — spec, design, implement, review, deploy — with review reject loop.',
    badge: 'community',
    accentColor: '#10b981',
    agents: [
      { name: 'Spec Writer', color: '#7c3aed' },
      { name: 'Designer', color: '#3b82f6' },
      { name: 'Implementer', color: '#3b82f6' },
      { name: 'Reviewer', color: '#10b981' },
      { name: 'Deployer', color: '#3b82f6' },
    ],
    load: () => gstack,
  },
  {
    id: 'shipflow-solo',
    name: 'Shipflow Solo',
    description: 'Minimal Ship setup — one human, one commander with all skills loaded.',
    badge: 'ship',
    accentColor: '#f59e0b',
    agents: [
      { name: 'Commander', color: '#f59e0b' },
    ],
    load: () => shipflowSolo,
  },
  {
    id: 'superpowers-solo',
    name: 'Superpowers Solo',
    description: 'Single Claude Code agent with full superpowers skill set.',
    badge: 'community',
    accentColor: '#7c3aed',
    agents: [
      { name: 'Claude Code', color: '#7c3aed' },
    ],
    load: () => superpowersSolo,
  },
  {
    id: 'blank',
    name: 'Blank Canvas',
    description: 'Start from scratch — empty canvas, add your own nodes and edges.',
    badge: null,
    accentColor: '#475569',
    agents: [],
    load: () => blank,
  },
]
