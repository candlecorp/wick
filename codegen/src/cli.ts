#!/usr/bin/env node

import { debug } from './common.js';
import yargs, { CommandModule } from 'yargs';

import * as rust from './languages/rust.js';
import * as json from './languages/json.js';

async function main() {
  yargs(process.argv.slice(2))
    .command({ command: rust.COMMAND_NAME, builder: rust.builder, describe: rust.desc } as CommandModule)
    .command({ command: json.COMMAND_NAME, builder: json.builder, describe: json.desc } as CommandModule)
    // .commandDir('languages', { extensions: ['ts', 'js'] })
    .demandCommand(1, 'You need to specify a language')
    .strictCommands()
    .help('h')
    .alias('h', 'help')
    .wrap(process.stdout.columns).argv;
}

debug('Starting');
main()
  .then(() => {
    debug('Done processing command');
    debug('Done with main');
  })
  .catch(e => {
    debug('Error %o', e);
    console.error(`Error running task : ${e}`);
    process.exit(1);
  });
