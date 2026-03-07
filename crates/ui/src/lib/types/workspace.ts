import { Workspace, WorkspaceType } from '@/bindings';

/**
 * Extended Workspace type that includes fields provided by the backend 
 * but missing from the basic Workspace binding, or specific to the UI's
 * runtime representation.
 */
export interface RuntimeWorkspace extends Workspace {
    workspace_type?: WorkspaceType;
    release_id?: string | null;
    last_activated_at?: string | null;
    context_hash?: string | null;
    config_generation?: number;
    compiled_at?: string | null;
    compile_error?: string | null;
}
