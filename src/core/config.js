const fs = require('fs-extra');
const path = require('path');

const CONFIG_DIR = '.vibe';
const FEATURES_DIR = path.join(CONFIG_DIR, 'features');
const CONFIG_FILE = path.join(CONFIG_DIR, 'config.json');

const ensureConfigDir = async () => {
  await fs.ensureDir(CONFIG_DIR);
  await fs.ensureDir(FEATURES_DIR);
};

const loadConfig = async () => {
  try {
    return await fs.readJson(CONFIG_FILE);
  } catch {
    return {
      aiProvider: 'anthropic',
      aiModel: 'claude-sonnet-4-20250514',
      language: 'javascript',
      testFramework: 'jest',
      testDir: 'src/__tests__',
      srcDir: 'src',
      apiKey: ''
    };
  }
};

const saveConfig = async (config) => {
  await fs.writeJson(CONFIG_FILE, config, { spaces: 2 });
};

module.exports = {
  CONFIG_DIR,
  FEATURES_DIR,
  ensureConfigDir,
  loadConfig,
  saveConfig,
};