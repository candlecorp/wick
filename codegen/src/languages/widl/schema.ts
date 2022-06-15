import yargs from 'yargs';
import {
  getTemplate,
  commitOutput,
  LANGUAGE,
  registerTypePartials,
  WIDL_TYPE,
  CommonOutputOptions,
  outputOpts,
} from '../../common';

import { registerHelpers } from 'widl-template';

const LANG = LANGUAGE.WIDL;
const TYPE = WIDL_TYPE.Schema;

export const command = `${TYPE} [options]`;

export const desc = 'Generate boilerplate WIDL schema for components';
export const builder = (yargs: yargs.Argv): yargs.Argv => {
  return yargs.options(outputOpts({})).example(`widl schemas/`, 'Prints generated interface schema to STDOUT');
};

export function handler(args: CommonOutputOptions): void {
  registerTypePartials(LANG, TYPE);

  registerHelpers();

  const template = getTemplate(LANG, TYPE);
  const generated = template({});
  commitOutput(generated, args.output, { force: args.force, silent: args.silent });
}
