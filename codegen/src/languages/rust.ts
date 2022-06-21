import yargs, { CommandModule } from 'yargs';
import { LANGUAGE } from '../common.js';

export const COMMAND_NAME = LANGUAGE.Rust;

import * as codegen_component from './rust/component.js';
import * as codegen_integration from './rust/integration.js';
import * as codegen_interface from './rust/interface.js';
import * as codegen_lib from './rust/wapc-lib.js';

export const desc = 'Generate interface code or JSON from Apex schemas';
export const builder = (yargs: yargs.Argv): yargs.Argv => {
  return yargs
    .command({
      command: codegen_component.command,
      builder: codegen_component.builder,
      describe: codegen_component.desc,
      handler: codegen_component.handler,
    } as CommandModule)
    .command({
      command: codegen_integration.command,
      builder: codegen_integration.builder,
      describe: codegen_integration.desc,
      handler: codegen_integration.handler,
    } as CommandModule)
    .command({
      command: codegen_interface.command,
      builder: codegen_interface.builder,
      describe: codegen_interface.desc,
      handler: codegen_interface.handler,
    } as CommandModule)
    .command({
      command: codegen_lib.command,
      builder: codegen_lib.builder,
      describe: codegen_lib.desc,
      handler: codegen_lib.handler,
    } as CommandModule)
    .demandCommand(1, 'You need to specify a type')
    .strictCommands()
    .help('h')
    .alias('h', 'help')
    .wrap(yargs.terminalWidth());
};
