import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import fs from 'fs-extra';
import path from 'path';
import os from 'os';
import * as project from './project';

describe('core/project', () => {
    let testDir: string;

    beforeEach(async () => {
        testDir = path.join(os.tmpdir(), `vibe-test-${Math.random().toString(36).slice(2)}`);
        await fs.ensureDir(testDir);
        vi.spyOn(process, 'cwd').mockReturnValue(testDir);
    });

    afterEach(async () => {
        await fs.remove(testDir);
        vi.restoreAllMocks();
    });

    describe('sanitizeFileName', () => {
        it('replaces slashes with underscores', () => {
            expect(project.sanitizeFileName('path/to/file.md')).toBe('path_to_file.md');
            expect(project.sanitizeFileName('path\\to\\file.md')).toBe('path_to_file.md');
        });
    });

    describe('initProject', () => {
        it('creates project directories and default files', async () => {
            await project.initProject();

            const expectedDir = path.join(testDir, '.ship');
            expect(await fs.pathExists(expectedDir)).toBe(true);
            expect(await fs.pathExists(path.join(expectedDir, 'ADR'))).toBe(true);
            expect(await fs.pathExists(path.join(expectedDir, 'Issues', 'backlog'))).toBe(true);
            expect(await fs.pathExists(path.join(expectedDir, 'README.md'))).toBe(true);
            expect(await fs.pathExists(path.join(expectedDir, 'log.md'))).toBe(true);
        });
    });

    describe('createIssue', () => {
        it('creates an issue with a generated stub template', async () => {
            await project.initProject();
            const filePath = await project.createIssue('My new issue', 'Test issue details');

            expect(await fs.pathExists(filePath)).toBe(true);
            expect(filePath).toContain('my-new-issue.md');
            expect(filePath).toContain('backlog');
        });

        it('throws error for invalid status', async () => {
            await project.initProject();
            await expect(project.createIssue('Title', 'Desc', 'invalid-status')).rejects.toThrow('Invalid status: invalid-status');
        });

        it('finds project dir upwards', async () => {
            await project.initProject();
            const subDir = path.join(testDir, 'some', 'deep', 'subfolder');
            await fs.ensureDir(subDir);
            vi.spyOn(process, 'cwd').mockReturnValue(subDir);

            const filePath = await project.createIssue('Deep issue', 'Details');
            expect(filePath).toContain(path.join(testDir, project.PROJECT_DIR_NAME));
        });
    });

    describe('createADR', () => {
        it('creates an ADR and writes it to the ADR directory', async () => {
            await project.initProject();
            const filePath = await project.createADR('Use Vitest', 'Decision is final');

            expect(await fs.pathExists(filePath)).toBe(true);
            expect(filePath).toContain('use-vitest.md');
            expect(filePath).toContain('ADR');
        });
    });
});
