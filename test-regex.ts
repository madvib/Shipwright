import * as fs from 'fs';

const FRONTMATTER_RE = /^\s*(\+\+\+|---)[ \t]*\r?\n([\s\S]*?)\r?\n[ \t]*\1[ \t]*(?:\r?\n|$)/;

const text = fs.readFileSync('.ship/project/releases/v0.1.0-alpha.md', 'utf8');

console.log('--- START ---');
const match = text.match(FRONTMATTER_RE);

if (match) {
    console.log('Match found!');
    console.log('Delimiter:', match[1]);
    console.log('Frontmatter:', JSON.stringify(match[2].substring(0, 50)) + '...');
} else {
    console.log('No match found.');
}
