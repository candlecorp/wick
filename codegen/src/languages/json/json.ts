import yargs from 'yargs';
import fs from 'fs-extra';
import toml from 'toml';
import {
  commitOutput,
  LANGUAGE,
  registerTypePartials,
  JSON_TYPE,
  outputOpts,
  widlOpts,
  CommonOutputOptions,
  CommonWidlOptions,
} from '../../common';

import { registerHelpers } from 'widl-template';

import { processDir } from '../../process-widl-dir';
import path from 'path';

const LANG = LANGUAGE.JSON;
const TYPE = JSON_TYPE.Interface;

export const command = `${TYPE} <name> <schema_dir> [options]`;

export const desc = 'Generate JSON representation of a WIDL file';
export const builder = (yargs: yargs.Argv): yargs.Argv => {
  return yargs
    .positional('name', {
      demandOption: true,
      type: 'string',
      description: 'Path to directory containing WIDL schema files',
    })
    .positional('schema_dir', {
      demandOption: true,
      type: 'string',
      description: 'Path to directory containing WIDL schema files',
    })
    .options(outputOpts(widlOpts({})))
    .example(`${TYPE} schemas/`, 'Prints JSON-ified schema to STDOUT');
};

export interface Arguments extends CommonOutputOptions, CommonWidlOptions {
  name: string;
  schema_dir: string;
}

export async function handler(args: Arguments): Promise<void> {
  registerTypePartials(LANG, TYPE);
  const options = {
    root: args.root,
  };
  registerHelpers(options);

  const version = (await cargoVersion(process.cwd())) || '';

  const collectionSignature = processDir(args.name, args.schema_dir);
  collectionSignature.version = version;

  const generated = JSON.stringify(collectionSignature, null, 2);

  commitOutput(generated, args.output, { force: args.force, silent: args.silent });
}

async function cargoVersion(dir: string): Promise<string | undefined> {
  const cargoPath = path.join(dir, 'Cargo.toml');
  const stat = await fs.stat(cargoPath);
  if (stat.isFile()) {
    const tomlSource = await fs.readFile(cargoPath, 'utf-8');
    const cargo = toml.parse(tomlSource);
    return cargo?.package?.version;
  } else {
    return undefined;
  }
}
