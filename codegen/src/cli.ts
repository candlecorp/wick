#!/usr/bin/env node

import { debug } from './common';
import yargs from 'yargs';

debug('Starting');
try {
  yargs(process.argv.slice(2))
    .commandDir('languages')
    .demandCommand(1, 'You need to specify a language')
    .strictCommands()
    .help('h')
    .alias('h', 'help')
    .wrap(yargs.terminalWidth()).argv;
  debug('Done processing command');
} catch (e) {
  debug('Error %o', e);
  console.error(`Error running task : ${e}`);
  process.exit(1);
}
debug('Done with main');
