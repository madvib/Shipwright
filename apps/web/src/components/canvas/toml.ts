import type { WorkflowRole, RoutingRule } from './types'

function tomlString(v: unknown): string {
  if (typeof v === 'string') return `"${v.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`
  if (typeof v === 'boolean') return v ? 'true' : 'false'
  if (typeof v === 'number') return String(v)
  return `"${v}"`
}

export function exportWorkflowToml(
  workflowName: string,
  roles: WorkflowRole[],
  routing: RoutingRule[],
): string {
  const lines: string[] = []

  lines.push('[workflow]')
  lines.push(`name = ${tomlString(workflowName)}`)
  lines.push('')

  for (const role of roles) {
    lines.push('[[roles]]')
    lines.push(`id = ${tomlString(role.id)}`)
    lines.push(`profile = ${tomlString(role.profile)}`)
    if (role.default) lines.push('default = true')
    if (role.description) lines.push(`description = ${tomlString(role.description)}`)
    lines.push('')
  }

  for (const rule of routing) {
    lines.push('[[routing]]')
    lines.push(`from = ${tomlString(rule.from)}`)
    lines.push(`job_kind = ${tomlString(rule.jobKind)}`)
    lines.push(`to = ${tomlString(rule.to)}`)
    lines.push(`gate = ${tomlString(rule.gate)}`)
    lines.push('')
  }

  return lines.join('\n')
}
