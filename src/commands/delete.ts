import inquirer from 'inquirer';
import chalk from 'chalk';
import fs from 'fs-extra';
import path from 'path';
import { ensureConfigDir, FEATURES_DIR } from '../core/config';

const deleteCommand = {
  command: 'delete <feature-id>',
  description: 'Delete a feature',
  action: async (featureId: string) => {
    await ensureConfigDir();

    const confirm = await inquirer.prompt([
      {
        type: 'confirm',
        name: 'confirmed',
        message: `Delete feature ${featureId}?`,
        default: false
      }
    ]);

    if (confirm.confirmed) {
      const featurePath = path.join(FEATURES_DIR, `${featureId}.json`);
      try {
        await fs.remove(featurePath);
        console.log(chalk.green(`✓ Feature deleted: ${featureId}`));
      } catch {
        console.log(chalk.red(`❌ Feature not found: ${featureId}`));
      }
    }
  }
};

export default deleteCommand;