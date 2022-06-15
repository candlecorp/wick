import yargs from 'yargs';
import { registerHelpers } from 'widl-template';
import {
  CODEGEN_TYPE,
  getTemplate,
  commitOutput,
  LANGUAGE,
  registerTypePartials,
  CommonOutputOptions,
  outputOpts,
  CommonWidlOptions,
  registerLanguageHelpers,
} from '../../common';

const LANG = LANGUAGE.Rust;
const TYPE = CODEGEN_TYPE.WapcLib;

export const command = `${TYPE}`;
export const desc = 'Generate the boilerplate lib.rs for WaPC components';

export const builder = (yargs: yargs.Argv): yargs.Argv => {
  return yargs.options(outputOpts({})).example(`${LANG} ${TYPE}`, 'Prints boilerplate lib.rs to STDOUT');
};

interface Arguments extends CommonWidlOptions, CommonOutputOptions {}

export function handler(args: Arguments): void {
  registerTypePartials(LANG, TYPE);
  registerLanguageHelpers(LANG);

  const options = {
    root: args.root,
  };
  registerHelpers(options);

  const template = getTemplate(LANG, TYPE);
  const generated = template({});

  commitOutput(generated, args.output, { force: args.force, silent: args.silent });
}
