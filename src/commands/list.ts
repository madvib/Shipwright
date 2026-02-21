import chalk from 'chalk';
import { ensureConfigDir } from '../core/config';
import { listFeatures } from '../core/features';
import { LANGUAGES } from '../constants';

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
      const statusColors: Record<string, Function> = {
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

export default listCommand;