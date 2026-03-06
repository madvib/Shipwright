import { ComponentType } from 'react';

export interface NavItem {
    id: string;
    path: string;
    label: string;
    icon: ComponentType<{ className?: string }>;
    priority?: 'primary' | 'secondary';
}

export interface NavSection {
    id: string;
    label: string;
    items: NavItem[];
}

export interface NavigationModule {
    id: string;
    sections: NavSection[];
}
