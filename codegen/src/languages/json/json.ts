import yargs, { ArgumentsCamelCase } from 'yargs';
import fs from 'fs-extra';
import toml from 'toml';
import {
  commitOutput,
  LANGUAGE,
  registerTypePartials,
  JSON_TYPE,
  outputOpts,
  parserOpts,
  CommonOutputOptions,
  CommonParserOptions,
  debug,
  readFile,
} from '../../common.js';

import { registerHelpers } from 'apex-template';

import { processDir } from '../../process-apex-dir.js';
import path from 'path';

export const LANG = LANGUAGE.JSON;
export const TYPE = JSON_TYPE.Interface;

export const command = `${TYPE} <name> <schema_dir> [options]`;

export const desc = 'Generate JSON representation of a Apex file';
export const builder = (yargs: yargs.Argv): yargs.Argv => {
  return yargs
    .positional('name', {
      demandOption: true,
      type: 'string',
      description: 'Path to directory containing Apex schema files',
    })
    .positional('schema_dir', {
      demandOption: true,
      type: 'string',
      description: 'Path to directory containing Apex schema files',
    })
    .options(
      outputOpts(
        parserOpts({
          stateful: { type: 'boolean' },
        }),
      ),
    )
    .example(`${TYPE} schemas/`, 'Prints JSON-ified schema to STDOUT');
};

export interface Arguments extends CommonOutputOptions, CommonParserOptions {
  name: string;
  stateful: boolean;
  schema_dir: string;
}

export async function handler(args: ArgumentsCamelCase<Arguments>): Promise<void> {
  await registerTypePartials(LANG, TYPE);

  const options = {
    root: args.root,
  };
  registerHelpers(options);

  const version = (await cargoVersion(process.cwd())) || '';

  const collectionSignature = await processDir(args.name, args.schema_dir, { stateful: args.stateful });

  collectionSignature.version = version;

  const generated = JSON.stringify(collectionSignature, null, 2);

  await commitOutput(generated, args.output, { force: args.force, silent: args.silent });
}

async function cargoVersion(dir: string): Promise<string | undefined> {
  const cargoPath = path.join(dir, 'Cargo.toml');
  const exists = await fs.pathExists(cargoPath);
  if (exists) {
    const tomlSource = await readFile(cargoPath);
    const cargo = toml.parse(tomlSource);
    return cargo?.package?.version;
  } else {
    return undefined;
  }
}
