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
  registerLanguageHelpers,
  readInterface,
} from '../../common.js';
import path from 'path';
import { BATCH_COMPONENT_NAME } from '../../batch_component.js';
import { CollectionSignature } from '../../types.js';

export const LANG = LANGUAGE.Rust;
export const TYPE = CODEGEN_TYPE.Component;

export const command = `${TYPE} <interface> [component_name] [options]`;

export const desc = 'Generate boilerplate for components';

export const builder = (yargs: yargs.Argv): yargs.Argv => {
  return yargs
    .positional('interface', {
      demandOption: true,
      type: 'string',
      description: "The path to the component's Apex schema",
    })
    .positional('component_name', {
      demandOption: false,
      type: 'string',
      description: 'The component to generate',
    })
    .options(
      outputOpts({
        all: { type: 'boolean', alias: 'a' },
        stateful: { type: 'boolean' },
        wellknown: { type: 'boolean' },
      }),
    )
    .example(`${LANG} ${TYPE} interface.json my_component`, 'Prints generated component code to STDOUT');
};

interface Arguments extends CommonOutputOptions {
  interface: string;
  component_name: string;
  stateful: boolean;
  all: boolean;
}

export async function handler(args: ArgumentsCamelCase<Arguments>): Promise<void> {
  await registerTypePartials(LANG, TYPE);
  registerLanguageHelpers(LANG);

  registerHelpers();

  const template = await getTemplate(LANG, TYPE);
  const [iface, ijson] = await readInterface(args.interface);
  const component_name = args.component_name;

  async function writeComponent(
    iface: CollectionSignature,
    component: string,
    fileName?: string,
    batch = false,
    wellknown = false,
  ) {
    const generated = template({
      name: component,
      interface: iface,
      stateful: args.stateful,
      wellknown,
      batch,
    });

    await commitOutput(generated, fileName, {
      force: args.force,
      silent: args.silent,
    });
  }

  if (!(args.all || component_name)) {
    throw new Error('Either component name or --all must be specified');
  }

  if (args.all) {
    if (component_name) {
      console.warn('Warning: component name (%s) and --all specified. --all takes precedence.');
    }
    for (const component in iface.components) {
      const fileName = component.replace(/[::]+/g, '/').replace(/[-]+/g, '_');
      await writeComponent(iface, component, path.join(args.output || '.', `${fileName}.rs`));
    }
    if (iface.wellknown) {
      for (const wellknown of iface.wellknown) {
        for (const component in wellknown.schema.components) {
          const fileName = component.replace(/[::]+/g, '/').replace(/[-]+/g, '_');
          await writeComponent(
            wellknown.schema,
            component,
            path.join(args.output || '.', `${fileName}.rs`),
            false,
            true,
          );
        }
      }
    }
    const fileName = path.join(args.output || '.', `${BATCH_COMPONENT_NAME}.rs`);
    await writeComponent(iface, BATCH_COMPONENT_NAME, fileName, true);
  } else if (component_name) {
    if (component_name == BATCH_COMPONENT_NAME) {
      await writeComponent(iface, BATCH_COMPONENT_NAME, args.output, true);
    } else {
      const component = iface.components[component_name];
      if (!component) {
        throw new Error(`Component name ${component_name} not found in interface`);
      }
      await writeComponent(iface, component_name, args.output);
    }
  }
}
