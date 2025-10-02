const chalk = require('chalk');
const { ensureConfigDir, loadConfig, saveConfig } = require('../core/config');

const configCommand = {
  command: 'config [key] [value]',
  description: 'View or update configuration',
  action: async (key, value) => {
    await ensureConfigDir();
    const config = await loadConfig();

    if (!key) {
      // Show all config
      console.log(chalk.blue('\n⚙️  Current Configuration:\n'));
      console.log(chalk.gray('AI Provider:'), config.aiProvider);
      console.log(chalk.gray('AI Model:'), config.aiModel);
      console.log(chalk.gray('Language:'), config.language);
      console.log(chalk.gray('Test Framework:'), config.testFramework);
      console.log(chalk.gray('Test Directory:'), config.testDir);
      console.log(chalk.gray('Source Directory:'), config.srcDir);
      console.log(chalk.gray('API Key:'), config.apiKey ? '***configured***' : 'not set');
      console.log();
      return;
    }

    if (!value) {
      // Show specific key
      console.log(chalk.gray(`${key}:`), config[key] || 'not set');
      return;
    }

    // Update key
    config[key] = value;
    await saveConfig(config);
    console.log(chalk.green(`✓ Updated ${key} to: ${value}`));
  }
};

module.exports = configCommand;