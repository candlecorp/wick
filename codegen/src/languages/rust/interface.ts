import yargs from 'yargs';
import { registerHelpers } from 'widl-template';
import {
  CODEGEN_TYPE,
  getTemplate,
  commitOutput,
  LANGUAGE,
  registerTypePartials,
  widlOpts,
  CommonWidlOptions,
  CommonOutputOptions,
  outputOpts,
  readInterface,
  registerLanguageHelpers,
} from '../../common';
import { BATCH_SIGNATURE } from '../../batch_component';

const LANG = LANGUAGE.Rust;
const TYPE = CODEGEN_TYPE.Interface;

export const command = `${TYPE} <interface> [options]`;
export const desc = 'Generate the Vino integration code for all component schemas';

export const builder = (yargs: yargs.Argv): yargs.Argv => {
  return yargs
    .positional('interface', {
      demandOption: true,
      type: 'string',
      description: 'Path to interface JSON',
    })
    .options(outputOpts(widlOpts({})))
    .example(`rust ${TYPE} interface.json`, 'Prints generated code to STDOUT');
};

interface Arguments extends CommonWidlOptions, CommonOutputOptions {
  interface: string;
}

export function handler(args: Arguments): void {
  registerTypePartials(LANG, TYPE);
  registerLanguageHelpers(LANG);

  const options = {
    root: args.root,
  };
  registerHelpers(options);

  const template = getTemplate(LANG, TYPE);
  const [iface, ijson] = readInterface(args.interface);

  const generated = template({
    interface: iface,
    batch: BATCH_SIGNATURE,
  });

  commitOutput(generated, args.output, { force: args.force, silent: args.silent });
}
