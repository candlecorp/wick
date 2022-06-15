import { expect } from 'chai';
import { describe } from 'mocha';
import path from 'path';
import fs from 'fs';
import os from 'os';

import { handler } from '../src/languages/widl/schema';

describe('command', function () {
  it('should generate default schemas', () => {
    const bignum = process.hrtime.bigint().toString();
    const filepath = path.join(os.tmpdir(), `${bignum}.txt`);

    handler({
      force: false,
      silent: false,
      output: filepath,
    });
    const contents = fs.readFileSync(filepath, 'utf-8');

    expect(contents).to.match(/Example inputs/);
  });
});
