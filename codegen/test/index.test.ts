import { expect } from 'chai';
import { describe } from 'mocha';
import path from 'path';
import fs from 'fs';
import os from 'os';

import { getTemplate, commitOutput, LANGUAGE } from '../src';
import { LANGUAGE_OFFERS } from '../src/common';

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

    it('should print to STDOUT when not passed a path', () => {
      commitOutput('file contents');
      expect(messages.length).to.equal(1);
      expect(messages[0]).to.equal('file contents');
    });
    it('should print to file when passed a new path', () => {
      const bignum = process.hrtime.bigint().toString();
      const filepath = path.join(os.tmpdir(), `${bignum}.txt`);
      expect(fs.existsSync(filepath)).to.be.false;
      const output = `Wrote "${bignum}" to file: ${filepath}\n`;
      commitOutput(output, filepath);
      expect(messages.length).to.equal(0);
      expect(fs.existsSync(filepath)).to.be.true;
      const realSrc = fs.readFileSync(filepath, 'utf-8');
      expect(realSrc).to.equal(output);
    });
    it('should not overwrite file without force', () => {
      const bignum = process.hrtime.bigint().toString();
      const filepath = path.join(os.tmpdir(), `${bignum}.txt`);
      expect(fs.existsSync(filepath)).to.be.false;
      fs.writeFileSync(filepath, 'content');
      expect(fs.existsSync(filepath)).to.be.true;
      const output = `Wrote "${bignum}" to file: ${filepath}\n`;
      function shouldNotThrow() {
        commitOutput(output, filepath);
      }
      expect(shouldNotThrow).to.not.throw();
      expect(messages.length).to.equal(0);
      expect(fs.existsSync(filepath)).to.be.true;
      const realSrc = fs.readFileSync(filepath, 'utf-8');
      expect(realSrc).to.equal('content');
    });
    it('should not throw if silenced', () => {
      const bignum = process.hrtime.bigint().toString();
      const filepath = path.join(os.tmpdir(), `${bignum}.txt`);
      expect(fs.existsSync(filepath)).to.be.false;
      fs.writeFileSync(filepath, 'content');
      expect(fs.existsSync(filepath)).to.be.true;
      const output = `Wrote "${bignum}" to file: ${filepath}\n`;
      function shouldThrow() {
        commitOutput(output, filepath, { silent: true });
      }
      expect(shouldThrow).to.not.throw();
      expect(messages.length).to.equal(0);
      expect(fs.existsSync(filepath)).to.be.true;
      const realSrc = fs.readFileSync(filepath, 'utf-8');
      expect(realSrc).to.equal('content');
    });
    it('should overwrite file when force is true', () => {
      const bignum = process.hrtime.bigint().toString();
      const filepath = path.join(os.tmpdir(), `${bignum}.txt`);
      expect(fs.existsSync(filepath)).to.be.false;
      fs.writeFileSync(filepath, 'content');
      expect(fs.existsSync(filepath)).to.be.true;
      const output = `Wrote "${bignum}" to file: ${filepath}\n`;
      function shouldNotThrow() {
        commitOutput(output, filepath, { force: true });
      }
      expect(shouldNotThrow).to.not.throw();
      expect(messages.length).to.equal(0);
      expect(fs.existsSync(filepath)).to.be.true;
      const realSrc = fs.readFileSync(filepath, 'utf-8');
      expect(realSrc).to.equal(output);
    });
    it('should overwrite file if the file is a stub', () => {
      const bignum = process.hrtime.bigint().toString();
      const filepath = path.join(os.tmpdir(), `${bignum}.txt`);
      expect(fs.existsSync(filepath)).to.be.false;
      fs.writeFileSync(filepath, '/* stub */');
      expect(fs.existsSync(filepath)).to.be.true;
      const output = `Wrote "${bignum}" to file: ${filepath}\n`;
      function shouldNotThrow() {
        commitOutput(output, filepath, { force: false });
      }
      expect(shouldNotThrow).to.not.throw();
      expect(messages.length).to.equal(0);
      expect(fs.existsSync(filepath)).to.be.true;
      const realSrc = fs.readFileSync(filepath, 'utf-8');
      expect(realSrc).to.equal(output);
    });
  });
});
