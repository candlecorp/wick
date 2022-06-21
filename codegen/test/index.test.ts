import chai, { expect } from 'chai';
import { describe } from 'mocha';
import path from 'path';
import fs from 'fs';
import os from 'os';
import chaiAsPromised from 'chai-as-promised';

chai.use(chaiAsPromised);

import { getTemplate, commitOutput, LANGUAGE } from '../src/index.js';
import { LANGUAGE_OFFERS } from '../src/common.js';
// shimming __dirname & __filename
import url from 'url';

const __filename = url.fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
// end

describe('getTemplate', function () {
  it('should get templates for every language and codegen type', () => {
    const emptyTemplate = [];
    for (const lang of Object.values(LANGUAGE)) {
      for (const type of Object.values(LANGUAGE_OFFERS[lang])) {
        const src = getTemplate(lang, type);
        if (!src) emptyTemplate.push(`${lang}/${type} missing`);
      }
    }
    expect(emptyTemplate).to.deep.equal([]);
  });
  describe('commitOutput', function () {
    const origLog = console.log;
    //@ts-ignore
    let messages: unknown[] = [];

    beforeEach(() => {
      messages = [];

      console.log = function (message?: unknown) {
        messages.push(message);
      };
    });

    afterEach(() => {
      console.log = origLog;
    });

    it('should print to STDOUT when not passed a path', async () => {
      await commitOutput('file contents');
      expect(messages.length).to.equal(1);
      //@ts-ignore
      expect(messages[0]).to.equal('file contents');
    });
    it('should print to file when passed a new path', async () => {
      const bignum = process.hrtime.bigint().toString();
      const filepath = path.join(os.tmpdir(), `${bignum}.txt`);
      expect(fs.existsSync(filepath)).to.be.false;
      const output = `Wrote "${bignum}" to file: ${filepath}\n`;
      await commitOutput(output, filepath);
      expect(messages.length).to.equal(0);
      expect(fs.existsSync(filepath)).to.be.true;
      const realSrc = fs.readFileSync(filepath, 'utf-8');
      expect(realSrc).to.equal(output);
    });
    it('should not overwrite file without force', async () => {
      const bignum = process.hrtime.bigint().toString();
      const filepath = path.join(os.tmpdir(), `${bignum}.txt`);
      expect(fs.existsSync(filepath)).to.be.false;
      fs.writeFileSync(filepath, 'content');
      expect(fs.existsSync(filepath)).to.be.true;
      const output = `Wrote "${bignum}" to file: ${filepath}\n`;
      async function shouldNotThrow() {
        await commitOutput(output, filepath);
      }
      await expect(shouldNotThrow()).to.not.be.rejected;
      expect(messages.length).to.equal(0);
      expect(fs.existsSync(filepath)).to.be.true;
      const realSrc = fs.readFileSync(filepath, 'utf-8');
      expect(realSrc).to.equal('content');
    });
    it('should not throw if silenced', async () => {
      const bignum = process.hrtime.bigint().toString();
      const filepath = path.join(os.tmpdir(), `${bignum}.txt`);
      expect(fs.existsSync(filepath)).to.be.false;
      fs.writeFileSync(filepath, 'content');
      expect(fs.existsSync(filepath)).to.be.true;
      const output = `Wrote "${bignum}" to file: ${filepath}\n`;
      async function shouldThrow() {
        await commitOutput(output, filepath, { silent: true });
      }
      await expect(shouldThrow()).to.not.be.rejected;
      expect(messages.length).to.equal(0);
      expect(fs.existsSync(filepath)).to.be.true;
      const realSrc = fs.readFileSync(filepath, 'utf-8');
      expect(realSrc).to.equal('content');
    });
    it('should overwrite file when force is true', async () => {
      const bignum = process.hrtime.bigint().toString();
      const filepath = path.join(os.tmpdir(), `${bignum}.txt`);
      expect(fs.existsSync(filepath)).to.be.false;
      fs.writeFileSync(filepath, 'content');
      expect(fs.existsSync(filepath)).to.be.true;
      const output = `Wrote "${bignum}" to file: ${filepath}\n`;
      async function shouldNotThrow() {
        await commitOutput(output, filepath, { force: true });
      }
      await expect(shouldNotThrow()).to.not.be.rejected;
      expect(messages.length).to.equal(0);
      expect(fs.existsSync(filepath)).to.be.true;
      const realSrc = fs.readFileSync(filepath, 'utf-8');
      expect(realSrc).to.equal(output);
    });
    it('should overwrite file if the file is a stub', async () => {
      const bignum = process.hrtime.bigint().toString();
      const filepath = path.join(os.tmpdir(), `${bignum}.txt`);
      expect(fs.existsSync(filepath)).to.be.false;
      fs.writeFileSync(filepath, '/* stub */');
      expect(fs.existsSync(filepath)).to.be.true;
      const output = `Wrote "${bignum}" to file: ${filepath}\n`;
      async function shouldNotThrow() {
        await commitOutput(output, filepath, { force: false });
      }
      await expect(shouldNotThrow()).to.not.be.rejected;
      expect(messages.length).to.equal(0);
      expect(fs.existsSync(filepath)).to.be.true;
      const realSrc = fs.readFileSync(filepath, 'utf-8');
      expect(realSrc).to.equal(output);
    });
  });
});
