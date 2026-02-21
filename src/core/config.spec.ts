import { describe, it, expect, vi, beforeEach } from 'vitest';
import fs from 'fs-extra';
import {
    CONFIG_DIR,
    FEATURES_DIR,
    CONFIG_FILE,
    ensureConfigDir,
    loadConfig,
    saveConfig,
    ProjectConfig
} from './config';

vi.mock('fs-extra');

describe('core/config', () => {
    beforeEach(() => {
        vi.resetAllMocks();
    });

    describe('ensureConfigDir', () => {
        it('ensures both config and features directories exist', async () => {
            await ensureConfigDir();
            expect(fs.ensureDir).toHaveBeenCalledWith(CONFIG_DIR);
            expect(fs.ensureDir).toHaveBeenCalledWith(FEATURES_DIR);
        });
    });

    describe('loadConfig', () => {
        it('returns the parsed json if file exists', async () => {
            const mockConfig = { aiProvider: 'openai' };
            vi.mocked(fs.readJson).mockResolvedValueOnce(mockConfig);

            const result = await loadConfig();

            expect(fs.readJson).toHaveBeenCalledWith(CONFIG_FILE);
            expect(result).toEqual(mockConfig);
        });

        it('returns default config if reading fails', async () => {
            vi.mocked(fs.readJson).mockRejectedValueOnce(new Error('File not found'));

            const result = await loadConfig();

            expect(result.language).toBe('javascript');
        });
    });

    describe('saveConfig', () => {
        it('writes the config to the config file', async () => {
            const config: ProjectConfig = {
                language: 'typescript',
                testFramework: 'vitest',
                testDir: 'tests',
                srcDir: 'src'
            };

            await saveConfig(config);

            expect(fs.writeJson).toHaveBeenCalledWith(CONFIG_FILE, config, { spaces: 2 });
        });
    });
});
