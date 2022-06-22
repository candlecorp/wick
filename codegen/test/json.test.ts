import { expect } from 'chai';
import { describe } from 'mocha';
import path from 'path';
import fs from 'fs';
import os from 'os';

import { handler } from '../src/languages/json/json.js';
import { CollectionSignature } from '../src/types.js';
// shimming __dirname & __filename
import url from 'url';

const __filename = url.fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
// end

describe('json command', function () {
  it('should generate default interface schema json', async () => {
    const root = path.join(__dirname, 'fixtures');
    const bignum = process.hrtime.bigint().toString();
    const filepath = path.join(os.tmpdir(), `${bignum}.json`);
    await handler({
      $0: '',
      _: [],
      force: false,
      name: 'test-name',
      root,
      schema_dir: root,
      silent: false,
      stateful: false,
      output: filepath,
    });
    const contents = fs.readFileSync(filepath, 'utf-8');
    const json = JSON.parse(contents);

    const expected: CollectionSignature = {
      name: 'test-name',
      version: '',
      format: 1,
      config: {},
      features: { stateful: false, streaming: false, version: 0 },
      wellknown: [],
      types: {
        HttpRequest: {
          type: 'struct',
          name: 'HttpRequest',
          fields: {
            url: { type: 'string' },
            method: { type: 'string' },
            link: { type: 'link', capability: 'http' },
          },
        },
        HttpResponse: {
          type: 'struct',
          name: 'HttpResponse',
          fields: {
            body: { type: 'string' },
            headers: { type: 'map', key: { type: 'string' }, value: { type: 'string' } },
          },
        },
      },
      components: {
        add: {
          name: 'add',
          inputs: {
            left: { type: 'i64' },
            right: { type: 'i64' },
          },
          outputs: {
            sum: { type: 'i64' },
          },
        },
        'hello-world': {
          name: 'hello-world',
          inputs: {
            messages: { type: 'list', element: { type: 'string' } },
          },
          outputs: {
            greeting: { type: 'string' },
          },
        },
        'http-request': {
          name: 'http-request',
          inputs: {
            request: { type: 'ref', ref: '#/types/HttpRequest' },
          },
          outputs: {
            response: { type: 'ref', ref: '#/types/HttpResponse' },
          },
        },
      },
    };

    expect(json).to.deep.equal(expected);
  });
});
