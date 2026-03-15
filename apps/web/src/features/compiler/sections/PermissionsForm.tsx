import { PermissionsEditor } from '@ship/ui'
import type { Permissions } from '@ship/ui'

interface Props {
  permissions: Permissions
  onChange: (p: Permissions) => void
}

export function PermissionsForm({ permissions, onChange }: Props) {
  return <PermissionsEditor permissions={permissions} onChange={onChange} />
}
