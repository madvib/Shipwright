import chalk from 'chalk';
import { ensureConfigDir } from '../core/config';
import { loadFeature, Feature } from '../core/features';
import { LANGUAGES } from '../constants';

const showCommand = {
  command: 'show <feature-id>',
  description: 'Show feature details',
  action: async (featureId: string) => {
    await ensureConfigDir();

    let feature: Feature;
    try {
      feature = await loadFeature(featureId);
    } catch {
      console.log(chalk.red(`❌ Feature not found: ${featureId}`));
      return;
    }

    const language = LANGUAGES[feature.language];

    console.log(chalk.blue(`\n📋 ${feature.title}\n`));
    console.log(chalk.gray(`ID: ${feature.id}`));
    console.log(chalk.gray(`Status: ${feature.status}`));
    console.log(chalk.gray(`Language: ${language.name}`));
    console.log(chalk.gray(`Test Framework: ${feature.testFramework}`));
    console.log();

    if (feature.description) {
      console.log(chalk.bold('Description:'));
      console.log(feature.description);
      console.log();
    }

    if (feature.acceptanceCriteria && feature.acceptanceCriteria.length > 0) {
      console.log(chalk.bold('Acceptance Criteria:'));
      feature.acceptanceCriteria.forEach((c: string, i: number) => {
        console.log(`  ${i + 1}. ${c}`);
      });
      console.log();
    }

    if (feature.generatedFiles && Object.keys(feature.generatedFiles).length > 0) {
      console.log(chalk.bold('Generated Files:'));
      for (const [type, filePath] of Object.entries(feature.generatedFiles)) {
        console.log(`  ${type}: ${chalk.cyan(filePath)}`);
      }
      console.log();
    }
  }
};

export default showCommand;