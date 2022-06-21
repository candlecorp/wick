import yargs, { ArgumentsCamelCase } from 'yargs';
import { registerHelpers } from 'apex-template';
import {
  CODEGEN_TYPE,
  getTemplate,
  commitOutput,
  LANGUAGE,
  registerTypePartials,
  CommonParserOptions,
  CommonOutputOptions,
  outputOpts,
  registerLanguageHelpers,
  readInterface,
} from '../../common.js';
import { BATCH_SIGNATURE } from '../../batch_component.js';

export const LANG = LANGUAGE.Rust;
export const TYPE = CODEGEN_TYPE.Integration;

export const command = `${TYPE} <interface> [options]`;
export const desc = 'Generate the Wasmflow integration code for the passed interface and type.';

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

interface Arguments extends CommonParserOptions, CommonOutputOptions {
  interface: string;
  stateful: boolean;
  wellknown: boolean;
}

export async function handler(args: ArgumentsCamelCase<Arguments>): Promise<void> {
  await registerTypePartials(LANG, TYPE);
  registerLanguageHelpers(LANG);

  const options = {
    root: args.root,
  };
  registerHelpers(options);

  const template = await getTemplate(LANG, TYPE);
  const [iface, ijson] = await readInterface(args.interface);

  const generated = template({
    interface: iface,
    interface_json: ijson,
    stateful: args.stateful,
    wellknown: args.wellknown,
    batch: BATCH_SIGNATURE,
  });

  await commitOutput(generated, args.output, { force: args.force, silent: args.silent });
}
