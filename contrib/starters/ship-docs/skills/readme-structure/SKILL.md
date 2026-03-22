---
name: README Structure
description: Writing effective READMEs with clear structure, quick start, and contributor guidance
tags: [documentation, readme, onboarding, contributing]
---

# README Structure

## Section Order

A README serves two audiences: users who want to use the project, and contributors who want to change it. Structure for users first, contributors second.

### Required Sections (in order)

1. **Title and description** (1-2 sentences)
2. **Quick Start** (copy-paste commands to get running)
3. **Installation** (detailed setup instructions)
4. **Usage** (common use cases with examples)
5. **Configuration** (environment variables, config files)
6. **Contributing** (how to submit changes)
7. **License**

### Optional Sections (add when relevant)

- Architecture overview (for complex projects)
- API reference (or link to separate docs)
- FAQ (for common questions that keep recurring)
- Changelog (or link to CHANGELOG.md)
- Acknowledgments

## Title and Description

```markdown
# Project Name

One sentence describing what this project does and who it's for.
A second sentence describing the key differentiator or approach.
```

Do not: use marketing language, list every feature, include badges before the description.

## Quick Start

The most important section. A new user should go from zero to running in under 2 minutes.

```markdown
## Quick Start

```bash
npm install my-project
```

```typescript
import { createClient } from "my-project";

const client = createClient({ apiKey: "your-key" });
const result = await client.analyze("Hello, world!");
console.log(result.sentiment); // "positive"
```
```

### Quick Start Rules

| Rule | Why |
|------|-----|
| Maximum 5 commands | More than 5 and people bail |
| Show the output | Proves it works, sets expectations |
| Use the simplest possible example | Complex examples belong in Usage |
| Include the import/require | Do not assume the reader knows the module name |

## Installation

Cover all supported methods. Be specific about versions and prerequisites.

```markdown
## Installation

### Prerequisites

- Node.js 18 or later
- PostgreSQL 15 or later

### npm

```bash
npm install my-project
```

### From source

```bash
git clone https://github.com/org/my-project.git
cd my-project
npm install
npm run build
```
```

## Usage

Show 3-5 common use cases. Each with a title, code example, and expected output.

```markdown
## Usage

### Basic Analysis

```typescript
const result = await client.analyze("The product is excellent");
// { sentiment: "positive", confidence: 0.95 }
```

### Batch Processing

```typescript
const results = await client.analyzeBatch([
  "Great service",
  "Terrible experience",
]);
// [{ sentiment: "positive", ... }, { sentiment: "negative", ... }]
```

### Error Handling

```typescript
try {
  const result = await client.analyze("");
} catch (error) {
  if (error instanceof ValidationError) {
    console.error("Invalid input:", error.message);
  }
}
```
```

## Configuration

Document every configuration option with its type, default, and description.

```markdown
## Configuration

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| API_KEY | string | (required) | Authentication key |
| TIMEOUT_MS | integer | 5000 | Request timeout in milliseconds |
| LOG_LEVEL | string | "info" | Logging level: debug, info, warn, error |
| MAX_RETRIES | integer | 3 | Number of retry attempts on failure |
```

Use a table format. Tables are scannable. Prose paragraphs describing config options are not.

## Contributing

Tell contributors exactly how to submit a change.

```markdown
## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/my-feature`)
3. Make your changes and add tests
4. Run the test suite (`npm test`)
5. Commit with a descriptive message (`git commit -m "feat: add batch analysis"`)
6. Push and open a pull request
```

### Contributing Rules

- State the commit message format (conventional commits, imperative tense)
- Mention required checks (tests, linting, formatting)
- Link to a CONTRIBUTING.md for detailed guidelines if they exist
- State the code of conduct expectation

## Common Mistakes

| Mistake | Problem | Fix |
|---------|---------|-----|
| Badges before description | Reader does not know what the project is | Description first, badges after |
| No quick start | Reader leaves before trying | Add 5-command quick start |
| Outdated examples | Examples throw errors | Test examples in CI |
| Assuming knowledge | "Just run the server" | Spell out every command |
| Wall of text | Nobody reads it | Use headings, tables, code blocks |
| Config in prose | Hard to scan | Use table format |

## README Maintenance Checklist

- [ ] Description is accurate and current
- [ ] Quick start works from a clean environment
- [ ] All code examples run without modification
- [ ] Configuration table lists every option
- [ ] Installation covers all supported methods
- [ ] Contributing section matches actual workflow
- [ ] No dead links
- [ ] No references to deprecated features
