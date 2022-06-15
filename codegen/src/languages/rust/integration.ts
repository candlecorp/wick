import yargs from 'yargs';
import { registerHelpers } from 'widl-template';
import {
  CODEGEN_TYPE,
  getTemplate,
  commitOutput,
  LANGUAGE,
  registerTypePartials,
  CommonWidlOptions,
  CommonOutputOptions,
  outputOpts,
  registerLanguageHelpers,
  readInterface,
} from '../../common';
import { BATCH_SIGNATURE } from '../../batch_component';

const LANG = LANGUAGE.Rust;
const TYPE = CODEGEN_TYPE.Integration;

export const command = `${TYPE} <interface> [options]`;
export const desc = 'Generate the Vino integration code for the passed interface and type.';

export const builder = (yargs: yargs.Argv): yargs.Argv => {
  return yargs
    .positional('interface', {
      demandOption: true,
      type: 'string',
      description: 'Path to interface JSON',
    })
    .options(
      outputOpts({
        stateful: { type: 'boolean' },
        wellknown: { type: 'boolean' },
      }),
    )
    .example(`rust ${TYPE} interface.json`, 'Prints generated code to STDOUT');
};

interface Arguments extends CommonWidlOptions, CommonOutputOptions {
  interface: string;
  stateful: boolean;
  wellknown: boolean;
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
    interface_json: ijson,
    stateful: args.stateful,
    wellknown: args.wellknown,
    batch: BATCH_SIGNATURE,
  });

  commitOutput(generated, args.output, { force: args.force, silent: args.silent });
}
