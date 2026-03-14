import { closest, levenshtein, search } from '../index.mjs';

const distance = levenshtein('kitten', 'sitting');
console.log(`levenshtein('kitten', 'sitting') = ${distance}`);
if (distance !== 3) {
  console.error(`Expected 3, got ${distance}`);
  process.exit(1);
}

const results = search('test', ['test', 'testing', 'best', 'unrelated']);
console.log(`search results: ${results.length} matches`);
if (results.length === 0) {
  console.error('Expected at least one search result');
  process.exit(1);
}

const match = closest('hello', ['hello world', 'help', 'hero']);
console.log(`closest('hello', ...) = ${match}`);
if (match === null) {
  console.error('Expected a match, got null');
  process.exit(1);
}

console.log('ESM import test passed');
