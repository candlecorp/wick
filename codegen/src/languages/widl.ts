import yargs from 'yargs';
import { LANGUAGE } from '../common';

const COMMAND_NAME = LANGUAGE.WIDL;

export const desc = 'Generate WIDL interfaces';
export const builder = (yargs: yargs.Argv): yargs.Argv => {
  return yargs
    .commandDir(COMMAND_NAME)
    .demandCommand(1, 'You need to specify a type')
    .strictCommands()
    .help('h')
    .alias('h', 'help')
    .wrap(yargs.terminalWidth());
};
