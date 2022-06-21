import yargs, { ArgumentsCamelCase } from 'yargs';
import { registerHelpers } from 'apex-template';
import {
  CODEGEN_TYPE,
  getTemplate,
  commitOutput,
  LANGUAGE,
  registerTypePartials,
  CommonOutputOptions,
  outputOpts,
  CommonParserOptions,
  registerLanguageHelpers,
} from '../../common.js';

export const LANG = LANGUAGE.Rust;
export const TYPE = CODEGEN_TYPE.WapcLib;

export const command = `${TYPE}`;
export const desc = 'Generate the boilerplate lib.rs for WaPC components';

export const builder = (yargs: yargs.Argv): yargs.Argv => {
  return yargs.options(outputOpts({})).example(`${LANG} ${TYPE}`, 'Prints boilerplate lib.rs to STDOUT');
};

interface Arguments extends CommonParserOptions, CommonOutputOptions {}

export async function handler(args: ArgumentsCamelCase<Arguments>): Promise<void> {
  await registerTypePartials(LANG, TYPE);
  registerLanguageHelpers(LANG);

  const options = {
    root: args.root,
  };
  registerHelpers(options);

  const template = await getTemplate(LANG, TYPE);
  const generated = template({});

  await commitOutput(generated, args.output, { force: args.force, silent: args.silent });
}
