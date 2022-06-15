import fs from 'fs';
import path from 'path';
import findroot from 'find-root';
import DEBUG from 'debug';
import { handlebars } from 'widl-template';
import { AbstractNode, Kind, ListType, MapType, Named, Optional } from '@wapc/widl/ast';
import yargs, { string } from 'yargs';
import { ComponentSignature, CollectionSignature } from './types';
import { snakeCase } from 'change-case-all';
export const debug = DEBUG('vino-codegen');

export enum LANGUAGE {
  Rust = 'rust',
  WIDL = 'widl',
  JSON = 'json',
}

export enum CODEGEN_TYPE {
  Component = 'component',
  Integration = 'integration',
  Interface = 'interface',
  WapcLib = 'wapc-lib',
}

export enum WIDL_TYPE {
  Interface = 'interface',
  Schema = 'schema',
}

export enum JSON_TYPE {
  Interface = 'interface',
}

export const LANGUAGE_OFFERS = {
  [LANGUAGE.Rust]: CODEGEN_TYPE,
  [LANGUAGE.WIDL]: WIDL_TYPE,
  [LANGUAGE.JSON]: JSON_TYPE,
};

export const DEFAULT_CODEGEN_TYPE = CODEGEN_TYPE.Integration;

export function readFile(path: string): string {
  try {
    return fs.readFileSync(path, 'utf-8');
  } catch (e: unknown) {
    throw new Error(`Could not read file at ${path}: ${e}`);
  }
}

export interface NormalizedFilename {
  raw: string;
  basename: string;
  withoutExtension: string;
  unhyphenated: string;
  extension: string;
}

export function normalizeFilename(filename: string): NormalizedFilename {
  const basename = path.basename(filename);
  const extension = path.extname(filename);
  const withoutExtension = basename.replace(extension, '');

  return {
    raw: filename,
    extension,
    basename,
    withoutExtension,
    unhyphenated: withoutExtension.replace('-', '_'),
  };
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function getTemplate(language: LANGUAGE, type: CODEGEN_TYPE | WIDL_TYPE | JSON_TYPE): (data: any) => string {
  const templatePath = path.join(findroot(__dirname), 'templates', language, `${type}.hbs`);
  debug('Reading template %o->%o located at %o', language, type, templatePath);
  debug('Compiling template from %o', templatePath);
  const template = handlebars.compile(readFile(templatePath));
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  return (data: any) => {
    try {
      debug.extend('trace')('Rendering template %o with data %o', path.basename(templatePath), data);
      return template(data);
    } catch (e) {
      console.error(`Error rendering template: ${e}`);
      throw e;
    }
  };
}

export function registerPartial(language: LANGUAGE, partial: string): void {
  const partialPath = path.join(findroot(__dirname), 'templates', language, 'partials', `${partial}.hbs`);
  debug('registering partial template %o->%o located at %o', language, partial, partialPath);
  const partialSource = readFile(partialPath);
  handlebars.registerPartial(partial, partialSource);
}

export function registerTypePartials(language: LANGUAGE, type: CODEGEN_TYPE | WIDL_TYPE | JSON_TYPE): void {
  registerCommonPartials(language);
  const relativeDir = path.join(language, 'partials', type);
  const dir = path.join(findroot(__dirname), 'templates', relativeDir);
  debug(`Looking for partials in %o`, dir);
  if (!fs.existsSync(dir)) return;
  const files = fs.readdirSync(dir);
  for (const file of files) {
    const name = file.replace(path.extname(`${file}.hbs`), '');
    const partialPath = path.join(dir, file);
    debug(`Loading partial %o`, partialPath);
    const exists = fs.existsSync(partialPath);
    if (exists) {
      debug(`Registering partial for %o:%o`, language, type);
      const partialSource = readFile(partialPath);
      handlebars.registerPartial(name, partialSource);
    }
  }
}

export interface HelperMap {
  [name: string]: Handlebars.HelperDelegate;
}

export function registerLanguageHelpers(lang: LANGUAGE): void {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  handlebars.registerHelper('ifEmpty', function (this: any, context: unknown, options): string {
    let isEmpty = false;
    if (context === undefined || context === null) {
      isEmpty = true;
    } else if (typeof context === 'string' && context.length === 0) {
      isEmpty = true;
    } else if (Array.isArray(context) && context.length === 0) {
      isEmpty = true;
    } else if (context && typeof context === 'object' && Object.keys(context).length === 0) {
      isEmpty = true;
    } else {
      isEmpty = false;
    }
    return isEmpty ? options.fn(this) : options.inverse(this);
  });
  handlebars.registerHelper('get', function (this: any, context: unknown, prop: string, options): any {
    if (typeof context !== 'object') {
      debug(`Called get helper to get property '${prop}' on a non-object primitive value`);
      return undefined;
    }
    if (context === null) {
      debug(`Called get helper to get property '${prop}' on a null value`);
      return undefined;
    } else {
      return (context as Record<string, any>)[prop];
    }
  });
  switch (lang) {
    case LANGUAGE.Rust:
      {
        handlebars.registerHelper('refToModulePath', function (context: string): string {
          if (context) {
            return context.substr(1).split('/').slice(1).join('::');
          } else {
            throw new Error(`Called refToModulePath with invalid context: ${context}`);
          }
        });

        // Returns a snakeCased version of a module path.
        handlebars.registerHelper('moduleName', function (context: string): string {
          if (context) {
            return context
              .split('::')
              .map(n => snakeCase(n))
              .join('::');
          } else {
            throw new Error(`Called moduleName with invalid context: ${context}`);
          }
        });

        // Returns a snakeCased version of the last part of a module path.
        handlebars.registerHelper('moduleLeafName', function (context: string): string {
          if (context) {
            const parts = context.split('::').map(n => snakeCase(n));
            return parts[parts.length - 1];
          } else {
            throw new Error(`Called moduleName with invalid context: ${context}`);
          }
        });

        // Takes a map of components and a root namespace and iterates over
        // the first layer of components in that namespace.
        // Useful when used recursively to traverse a list of components in a
        // hierarchical fashion.
        handlebars.registerHelper(
          'eachNamespace',
          function (context: {
            hash: {
              components: Record<string, ComponentSignature>;
              namespace?: string;
            };
            fn: Handlebars.TemplateDelegate;
          }): string {
            let returnText = '';
            interface NamespaceEntry {
              namespace?: string;
              components: Record<string, ComponentSignature>;
            }
            const namespaces: Record<string, NamespaceEntry> = {};
            const SEP = '::';
            const rootNs = context.hash.namespace || '';

            const rootNsParts = rootNs ? rootNs.split(SEP) : [];
            context.hash.namespace;
            for (const name in context.hash.components) {
              const component = context.hash.components[name] as ComponentSignature;

              const nsPathParts = name.split(SEP).slice(0, -1);
              const componentNamespace = nsPathParts.join(SEP);

              if (componentNamespace.startsWith(rootNs)) {
                const remainingPath = nsPathParts.slice(rootNsParts.length);
                const currentLevel = remainingPath[0] || '';
                if (namespaces[currentLevel]) {
                  namespaces[currentLevel].components[name] = component;
                } else {
                  namespaces[currentLevel] = {
                    namespace: currentLevel,
                    components: { [name]: component },
                  };
                }
              }
            }

            for (const nsName in namespaces) {
              const entry = namespaces[nsName];
              debug('NS entry: %o with %o components', nsName, Object.keys(entry.components).length);
              const inner = context.fn(entry);
              returnText += inner;
            }

            return returnText;
          },
        );
      }
      break;
    default:
  }
}

export function registerCommonPartials(language: LANGUAGE): void {
  const relativeDir = path.join(language, 'partials', 'common');
  const dir = path.join(findroot(__dirname), 'templates', relativeDir);
  debug(`Looking for partials in %o`, dir);
  if (!fs.existsSync(dir)) return;
  const files = fs.readdirSync(dir);
  for (const file of files) {
    const name = file.replace(path.extname(`${file}.hbs`), '');
    const partialPath = path.join(dir, file);
    debug(`Loading partial %o`, partialPath);
    const exists = fs.existsSync(partialPath);
    if (exists) {
      debug(`Registering common partial for %o: %o`, language, name);
      const partialSource = readFile(partialPath);
      handlebars.registerPartial(name, partialSource);
    }
  }
}

// This should be a separate module but won't until it does a complete codegen
export function codegen(node: AbstractNode): string {
  switch (node.kind) {
    case Kind.Named:
      return (<Named>node).name.value;
    case Kind.Optional:
      return `${codegen((<Optional>node).type as unknown as AbstractNode)}?`;
    case Kind.MapType:
      return `{${codegen((<MapType>node).keyType as unknown as AbstractNode)}:${codegen(
        (<MapType>node).valueType as unknown as AbstractNode,
      )}`;
    case Kind.ListType:
      return `[${codegen((<ListType>node).type as unknown as AbstractNode)}]`;
    default:
      // console.log(node);
      throw new Error(`Unhandled node ${node.kind}`);
  }
}

export interface CommonOutputOptions {
  force: boolean;
  silent: boolean;
  output?: string;
}

export interface CommonWidlOptions {
  root: string;
}

export function outputOpts(obj: { [key: string]: yargs.Options }): typeof obj {
  if (typeof obj != 'object' || obj === null) throw new Error(`Invalid argument: ${obj}`);
  const commonOptions = {
    s: {
      alias: 'silent',
      describe: 'Silently ignore write errors',
      type: 'boolean',
    },
    f: {
      alias: 'force',
      describe: 'Overwrite destination output even if it exists',
      type: 'boolean',
      implies: 'o',
    },
    o: {
      alias: 'output',
      describe: 'The output file or directory (defaults to STDOUT for text)',
      default: undefined,
      type: 'string',
    },
  };
  return Object.assign({}, obj, commonOptions);
}

export function widlOpts(obj: { [key: string]: yargs.Options }): typeof obj {
  if (typeof obj != 'object' || obj === null) throw new Error(`Invalid argument: ${obj}`);
  const commonOptions = {
    r: {
      alias: 'root',
      describe: 'The root directory to use when resolving import definitions',
      type: 'string',
    },
  };
  return Object.assign({}, obj, commonOptions);
}

interface CommitOptions {
  force?: boolean;
  silent?: boolean;
}

// eslint-disable-next-line @typescript-eslint/explicit-module-boundary-types
export function commitOutput(src: string, filePath?: string, options: CommitOptions = {}): void {
  if (filePath) {
    const basedir = path.dirname(filePath);
    if (!fs.existsSync(basedir)) {
      debug('Directory "%o" does not exist, creating it.', basedir);
      fs.mkdirSync(basedir);
    }
    if (fs.existsSync(filePath)) {
      if (options.force) {
        debug(`%o exists, overwriting anyway because of --force`, filePath);
      } else {
        const contents = fs.readFileSync(filePath, 'utf-8');
        if (contents.startsWith('/* stub */')) {
          debug(`%o exists but is a stub file, overwriting`, filePath);
        } else {
          debug(`Refusing to overwrite %o`, filePath);
          if (options.silent) return;
          else {
            debug(`%o exists, to overwrite pass --force to the codegen or delete the file`, filePath);
            return;
          }
        }
      }
    }
    debug(`Write to %o`, filePath);
    try {
      fs.writeFileSync(filePath, src);
    } catch (e) {
      console.error(`Error writing output to ${filePath}: `, e);
      throw e;
    }
  } else {
    console.log(src);
  }
}

export function readInterface(interfacePath: string): [CollectionSignature, string] {
  debug('Reading interface JSON at %o', interfacePath);
  const ifaceJson = fs.readFileSync(interfacePath, 'utf-8');
  const iface = JSON.parse(ifaceJson) as CollectionSignature;
  return [iface, ifaceJson];
}
