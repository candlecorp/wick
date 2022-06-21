import { readFile, readFileSync } from './common.js';
import path from 'path';
import fs from 'fs-extra';
import { parse, ast } from '@apexlang/core';
import { debug } from './common.js';

import {
  ComponentSignature,
  EnumSignature,
  EnumVariant,
  isApexType,
  CollectionSignature,
  RootType,
  StructSignature,
  FieldMap,
  TypeSignature,
} from './types.js';

export async function processDir(name: string, dir: string): Promise<CollectionSignature> {
  const { components, types, configs } = await processSchemaDir(dir);

  const collectionSignature: CollectionSignature = {
    name,
    version: '1.0.0',
    format: 1,
    wellknown: [],
    types: Object.fromEntries(types.map(t => [t.name, t])),
    components: Object.fromEntries(components.map(c => [c.name, c])),
    config: configs,
  };
  return collectionSignature;
}

async function processSchemaDir(
  dir: string,
  prefixes: string[] = [],
): Promise<{ types: RootType[]; components: ComponentSignature[]; configs: Record<string, StructSignature> }> {
  debug(`processing schema directory '${dir}'`);
  const rv = {
    types: [] as RootType[],
    components: [] as ComponentSignature[],
    configs: {} as Record<string, StructSignature>,
  };
  const entries = await fs.readdir(dir);

  const apexFiles = entries.filter(file => file.endsWith('.apex'));
  // Todo fetch and validate remote schema JSON somehow
  // const remoteIncludes = entries.filter(file => file.match('include.json'));
  const directories = entries.filter(file => fs.statSync(path.join(dir, file)).isDirectory());

  for (const subdir of directories) {
    const result = await processSchemaDir(path.join(dir, subdir), prefixes.concat([subdir]));
    rv.types.push(...result.types);
    result.components.forEach(comp => (comp.name = prefixes.concat([comp.name]).join('::')));
    rv.components.push(...result.components);
  }

  const components: ComponentSignature[] = [];
  const types: RootType[] = [];

  const resolver = (location: string) => {
    const pathParts = location.split('/');
    const importPath = path.join(dir, ...pathParts);
    const src = readFileSync(importPath);
    return src;
  };

  for (const file of apexFiles) {
    const apexSrc = await readFile(path.join(dir, file));
    const tree = parse(apexSrc, resolver);
    const [component, additionalTypes, config] = interpret(tree);
    types.push(...additionalTypes);
    components.push(component);
    if (config) {
      rv.configs[component.name] = config;
    }
  }
  rv.components.push(...components);
  rv.types.push(...types);

  return rv;
}

function getAnnotation(name: string, annotations: ast.Annotation[]): ast.Annotation | undefined {
  const result = annotations.filter(a => a.name.value == name)[0];
  return result;
}

function reduceType(type: ast.Type, annotations: ast.Annotation[] = []): TypeSignature {
  switch (type.getKind()) {
    case ast.Kind.Named: {
      const t = type as ast.Named;
      const name = t.name;
      if (isApexType(name.value)) {
        return { type: name.value };
      } else {
        const link = getAnnotation('capability', annotations);
        if (name.value === 'link') {
          if (link) {
            const collection = link.arguments[0];
            return {
              type: 'link',
              capability: collection.value.getValue(),
            };
          } else {
            return {
              type: 'link',
            };
          }
        } else if (name.value === 'struct') {
          // TODO: convert this to core Apex type once apex can represent it (see also: https://github.com/wapc/cli/issues/8)
          return { type: 'struct' };
        } else {
          return { type: 'ref', ref: `#/types/${name.value}` };
        }
      }
    }
    case ast.Kind.MapType: {
      const t = type as ast.MapType;
      return {
        type: 'map',
        key: reduceType(t.keyType, annotations),
        value: reduceType(t.valueType, annotations),
      };
    }
    case ast.Kind.ListType: {
      const t = type as ast.ListType;
      return {
        type: 'list',
        element: reduceType(t.type, annotations),
      };
    }
    case ast.Kind.Optional: {
      const t = type as ast.Optional;
      return {
        type: 'optional',
        option: reduceType(t.type, annotations),
      };
    }
  }
  throw new Error(`Unhandled type: ${type.getKind()}`);
}

function interpret(doc: ast.Document): [ComponentSignature, RootType[], StructSignature?] {
  const types = doc.definitions.filter(isType);
  const input_def = findByName(types, /inputs/i);
  const output_def = findByName(types, /outputs/i);
  const config_def = findByName(types, /config/i);
  const namespace = doc.definitions.find(def => def.isKind(ast.Kind.NamespaceDefinition));
  if (!namespace) throw new Error('Component schemas must define a namespace to use as the component name');
  if (!input_def) throw new Error('Component schemas must include a type definition named "Inputs"');
  if (!output_def) throw new Error('Component schemas must include a type definition named "Outputs"');

  const inputs: FieldMap = Object.fromEntries(
    input_def.fields.map(field => {
      return [field.name.value, reduceType(field.type, field.annotations)];
    }),
  );

  const outputs = Object.fromEntries(
    output_def.fields.map(field => {
      return [field.name.value, reduceType(field.type, field.annotations)];
    }),
  );

  const config = config_def ? reduceTypeDefinition(config_def) : undefined;

  const component: ComponentSignature = {
    name: (namespace as ast.NamespaceDefinition).name.value,
    inputs,
    outputs,
  };

  const typeSignatures = doc.definitions
    .filter(isSupportedType)
    .filter(t => !t.name.value.match(/inputs/i) && !t.name.value.match(/outputs/i) && !t.name.value.match(/config/i))
    .map(t => {
      switch (t.kind) {
        case ast.TypeDefinition.name:
          return reduceTypeDefinition(t as ast.TypeDefinition);
        case ast.EnumDefinition.name:
          return reduceEnumDefinition(t as ast.EnumDefinition);
        default:
          throw new Error(`Type ${t.kind} not yet handled`);
      }
    });
  return [component, typeSignatures, config];
}

function isType(def: ast.Definition): def is ast.TypeDefinition {
  return def.isKind(ast.Kind.TypeDefinition);
}

function isSupportedType(def: ast.Definition): def is ast.TypeDefinition | ast.EnumDefinition {
  return def.isKind(ast.Kind.TypeDefinition) || def.isKind(ast.Kind.EnumDefinition);
}

interface HasName extends ast.AbstractNode {
  name: { value: string };
}

function findByName<T extends HasName>(defs: T[], name: string | RegExp): T | undefined {
  return defs.find(def => def.name.value.match(name));
}

function reduceTypeDefinition(def: ast.TypeDefinition): StructSignature {
  const fields: Record<string, TypeSignature> = {};
  for (const field of def.fields) {
    fields[field.name.value] = reduceType(field.type, field.annotations);
  }

  return {
    type: 'struct',
    name: def.name.value,
    fields,
  };
}

function reduceEnumDefinition(def: ast.EnumDefinition): EnumSignature {
  const values: EnumVariant[] = [];
  for (const field of def.values) {
    values.push({ name: field.name.value, index: field.index.value });
  }

  return {
    type: 'enum',
    name: def.name.value,
    values,
  };
}
