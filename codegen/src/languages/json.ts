import yargs, { CommandModule } from 'yargs';
import { LANGUAGE } from '../common.js';

export const COMMAND_NAME = LANGUAGE.JSON;

import * as codegen_interface from './json/json.js';

export const desc = 'Generate JSON data';
export const builder = (yargs: yargs.Argv): yargs.Argv => {
  return yargs
    .command({
      command: codegen_interface.command,
      builder: codegen_interface.builder,
      describe: codegen_interface.desc,
      handler: codegen_interface.handler,
    } as CommandModule)
    .demandCommand(1, 'You need to specify a type')
    .strictCommands()
    .help('h')
    .alias('h', 'help')
    .wrap(yargs.terminalWidth());
};
