import { Workspace } from '@/bindings';

/**
 * Extended Workspace type that includes fields provided by the backend 
 * but missing from the basic Workspace binding, or specific to the UI's
 * runtime representation.
 */
export interface RuntimeWorkspace extends Workspace {
  release_id?: string | null;
  mcp_servers?: string[];
  skills?: string[];
}
