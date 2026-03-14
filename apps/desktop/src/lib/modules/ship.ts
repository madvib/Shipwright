import {
    ADRS_ROUTE,
    FEATURES_ROUTE,
    NOTES_ROUTE,
    OVERVIEW_ROUTE,
    RELEASES_ROUTE,
    WORKFLOW_WORKSPACE_ROUTE,
} from '../constants/routes';
import {
    FileStack,
    Flag,
    LayoutDashboard,
    NotebookPen,
    Package,
    Workflow,
} from 'lucide-react';
import { NavSection } from '../types/navigation';

export const SHIP_MODULE_ID = 'ship';

export const SHIP_NAV_SECTIONS: NavSection[] = [
    {
        id: 'overview',
        label: '',
        items: [
            { id: 'overview', path: OVERVIEW_ROUTE, label: 'Overview', icon: LayoutDashboard },
        ],
    },
    {
        id: 'workspaces',
        label: '',
        items: [
            { id: 'workspaces', path: WORKFLOW_WORKSPACE_ROUTE, label: 'Workspaces', icon: Workflow },
        ],
    },
    {
        id: 'project-docs',
        label: '',
        items: [
            { id: 'features', path: FEATURES_ROUTE, label: 'Features', icon: Flag },
            { id: 'releases', path: RELEASES_ROUTE, label: 'Releases', icon: Package },
            { id: 'decisions', path: ADRS_ROUTE, label: 'Decisions', icon: FileStack },
            { id: 'notes', path: NOTES_ROUTE, label: 'Notes', icon: NotebookPen },
        ],
    },
];
