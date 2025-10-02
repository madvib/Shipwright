const inquirer = require('inquirer');
const chalk = require('chalk');
const fs = require('fs-extra');
const path = require('path');
const { ensureConfigDir, FEATURES_DIR } = require('../core/config');

const deleteCommand = {
  command: 'delete <feature-id>',
  description: 'Delete a feature',
  action: async (featureId) => {
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

module.exports = deleteCommand;