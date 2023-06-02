#!/usr/bin/env ts-node

import { statSync } from 'fs';

// Test to see if a passed file is newer than another file

const target = process.argv[2];
const sources = process.argv.slice(3);

const targetStat = statSync(target);

for (const source of sources) {
  if (isNewer(source, target)) {
    console.log('true');
    process.exit(0);
  }
}

console.log('false');

function isNewer(source: string, target: string): boolean {
  const sourceStat = statSync(source);

  return sourceStat.mtimeMs > targetStat.mtimeMs;
}
