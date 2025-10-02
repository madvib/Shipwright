const chalk = require('chalk');
const { ensureConfigDir } = require('../core/config');
const { listFeatures } = require('../core/features');
const { LANGUAGES } = require('../constants');

const listCommand = {
  command: 'list',
  description: 'List all features',
  action: async () => {
    await ensureConfigDir();
    const features = await listFeatures();

    if (features.length === 0) {
      console.log(chalk.yellow('No features found. Create one with: vibe new "Feature Name"'));
      return;
    }

    console.log(chalk.blue('\n📋 Features:\n'));

    for (const feature of features) {
      const statusColors = {
        'documented': chalk.gray,
        'tests-generated': chalk.blue,
        'implemented': chalk.green
      };
      const statusColor = statusColors[feature.status] || chalk.gray;
      const langName = LANGUAGES[feature.language]?.name || feature.language;

      console.log(`${chalk.bold(feature.title)} ${statusColor(`[${feature.status}]`)} ${chalk.cyan(`(${langName})`)}`);
      console.log(`  ${chalk.gray(`ID: ${feature.id}`)}`);
      if (feature.description) {
        console.log(`  ${chalk.gray(feature.description)}`);
      }
      console.log();
    }
  }
};

module.exports = listCommand;