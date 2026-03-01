import { createFileRoute } from '@tanstack/react-router';
import NotesPage from '@/features/planning/NotesPage';

export const Route = createFileRoute('/project/notes')({
    component: NotesPage,
});
