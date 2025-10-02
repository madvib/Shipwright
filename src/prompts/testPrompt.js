const { LANGUAGES } = require('../constants');

const generateTestPrompt = (feature, language, testFramework) => {
  const langConfig = LANGUAGES[language];

  return `You are an expert ${langConfig.name} developer. Generate comprehensive test cases for the following feature using ${testFramework}:

Title: ${feature.title}
Description: ${feature.description}

Acceptance Criteria:
${feature.acceptanceCriteria.map((c, i) => `${i + 1}. ${c}`).join('\n')}

Requirements:
1. Test all acceptance criteria thoroughly
2. Include edge cases and error handling
3. Use proper ${testFramework} syntax and best practices for ${langConfig.name}
4. Write clear, descriptive test names
5. Include necessary setup/teardown
6. Follow ${langConfig.name} coding conventions

Respond ONLY with the complete test code. No explanations, no markdown formatting, just the raw ${langConfig.name} code.`;
};

module.exports = {
  generateTestPrompt,
};