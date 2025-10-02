const { LANGUAGES } = require('../constants');

const generateCodePrompt = (feature, language, testCode) => {
  const langConfig = LANGUAGES[language];

  return `You are an expert ${langConfig.name} developer. Generate the implementation code for the following feature that will pass these tests:

Title: ${feature.title}
Description: ${feature.description}

Acceptance Criteria:
${feature.acceptanceCriteria.map((c, i) => `${i + 1}. ${c}`).join('\n')}

Test Code:
\`\`\`${language}
${testCode}
\`\`\`

Requirements:
1. Pass all tests above
2. Follow ${langConfig.name} best practices and idioms
3. Include comprehensive documentation/comments
4. Handle edge cases properly
5. Write clean, maintainable, production-ready code
6. Use appropriate ${langConfig.name} design patterns

Respond ONLY with the complete implementation code. No explanations, no markdown formatting, just the raw ${langConfig.name} code.`;
};

module.exports = {
  generateCodePrompt,
};