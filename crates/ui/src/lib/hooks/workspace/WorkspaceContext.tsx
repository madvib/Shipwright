import { ReactNode, createContext, useContext } from 'react';
import { useWorkspaceController } from '../useWorkspaceController';

type WorkspaceController = ReturnType<typeof useWorkspaceController>;

const WorkspaceContext = createContext<WorkspaceController | null>(null);

export function WorkspaceProvider({ children }: { children: ReactNode }) {
  const workspace = useWorkspaceController();
  return <WorkspaceContext.Provider value={workspace}>{children}</WorkspaceContext.Provider>;
}

export function useWorkspace() {
  const context = useContext(WorkspaceContext);
  if (!context) {
    throw new Error('useWorkspace must be used within WorkspaceProvider');
  }
  return context;
}
