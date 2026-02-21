import chalk from 'chalk';
import { ensureConfigDir, loadConfig, saveConfig, ProjectConfig } from '../core/config';

const configCommand = {
  command: 'config [key] [value]',
  description: 'View or update configuration',
  action: async (key?: keyof ProjectConfig, value?: string) => {
    await ensureConfigDir();
    const config = await loadConfig();

    if (!key) {
      // Show all config
      console.log(chalk.blue('\n⚙️  Current Configuration:\n'));
      console.log(chalk.gray('Language:'), config.language);
      console.log(chalk.gray('Test Framework:'), config.testFramework);
      console.log(chalk.gray('Test Directory:'), config.testDir);
      console.log(chalk.gray('Source Directory:'), config.srcDir);
      console.log();
      return;
    }

    if (!value) {
      // Show specific key
      console.log(chalk.gray(`${key}:`), config[key] || 'not set');
      return;
    }

    // Update key
    (config as any)[key] = value;
    await saveConfig(config);
    console.log(chalk.green(`✓ Updated ${key} to: ${value}`));
  }
};

export default configCommand;