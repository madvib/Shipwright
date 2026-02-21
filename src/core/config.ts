import fs from 'fs-extra';
import path from 'path';

import os from 'os';

export const CONFIG_DIR_NAME = '.ship';

// Helper to get the correct config directory (local .ship or global ~/.ship)
const resolveConfigDir = (): string => {
  // If we are in a project that has a .ship folder, use it
  const localDir = path.join(process.cwd(), CONFIG_DIR_NAME);
  if (fs.existsSync(localDir)) {
    return localDir;
  }
  // Otherwise default to global
  return path.join(os.homedir(), CONFIG_DIR_NAME);
};

export const CONFIG_DIR = resolveConfigDir();
export const FEATURES_DIR = path.join(CONFIG_DIR, 'features');
export const CONFIG_FILE = path.join(CONFIG_DIR, 'config.json');

export interface ProjectConfig {
  language: string;
  testFramework: string;
  testDir: string;
  srcDir: string;
}

export const ensureConfigDir = async (): Promise<void> => {
  await fs.ensureDir(CONFIG_DIR);
  await fs.ensureDir(FEATURES_DIR);
};

export const loadConfig = async (): Promise<ProjectConfig> => {
  try {
    return await fs.readJson(CONFIG_FILE);
  } catch {
    return {
      language: 'javascript',
      testFramework: 'jest',
      testDir: 'src/__tests__',
      srcDir: 'src'
    };
  }
};

export const saveConfig = async (config: ProjectConfig): Promise<void> => {
  await fs.writeJson(CONFIG_FILE, config, { spaces: 2 });
};