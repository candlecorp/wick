#!/usr/bin/env ts-node

import { readFileSync } from 'fs';

const template_path = process.argv[2];

const imports = process.argv.slice(3);

const inner_json = imports.map((path) =>
  JSON.parse(readFileSync(path, 'utf-8'))
);

let template_json = JSON.parse(readFileSync(template_path, 'utf-8'));

let entries = inner_json.flatMap((json) => Object.entries(json));

for (const [key, value] of entries) {
  template_json['$defs'][key] = value;
}

console.log(JSON.stringify(template_json, null, 2));
