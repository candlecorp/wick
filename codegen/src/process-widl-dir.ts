import { readFile } from './common';
import path from 'path';
import fs from 'fs';
import { parse } from '@wapc/widl';
import {
  AbstractNode,
  Annotation,
  Definition,
  Document,
  EnumDefinition,
  Kind,
  ListType,
  MapType,
  Named,
  NamespaceDefinition,
  Optional,
  Type,
  TypeDefinition,
} from '@wapc/widl/ast';

import {
  ComponentSignature,
  EnumSignature,
  EnumVariant,
  isWidlType,
  CollectionSignature,
  RootType,
  StructSignature,
  FieldMap,
  TypeSignature,
} from './types';
import { string } from 'yargs';

export function processDir(name: string, dir: string): CollectionSignature {
  const { components, types, configs } = processSchemaDir(dir);

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

function processSchemaDir(
  dir: string,
  prefixes: string[] = [],
): { types: RootType[]; components: ComponentSignature[]; configs: Record<string, StructSignature> } {
  const rv = {
    types: [] as RootType[],
    components: [] as ComponentSignature[],
    configs: {} as Record<string, StructSignature>,
  };
  const entries = fs.readdirSync(dir);

  const widlFiles = entries.filter(file => file.endsWith('.widl'));
  // Todo fetch and validate remote schema JSON somehow
  // const remoteIncludes = entries.filter(file => file.match('include.json'));
  const directories = entries.filter(file => fs.statSync(path.join(dir, file)).isDirectory());

  for (const subdir of directories) {
    const result = processSchemaDir(path.join(dir, subdir), prefixes.concat([subdir]));
    rv.types.push(...result.types);
    result.components.forEach(comp => (comp.name = prefixes.concat([comp.name]).join('::')));
    rv.components.push(...result.components);
  }

  const components: ComponentSignature[] = [];
  const types: RootType[] = [];

  const resolver = (location: string) => {
    const pathParts = location.split('/');
    const importPath = path.join(dir, ...pathParts);
    const src = readFile(importPath);
    return src;
  };

  for (const file of widlFiles) {
    const widlSrc = readFile(path.join(dir, file));
    const tree = parse(widlSrc, resolver);
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

function getAnnotation(name: string, annotations: Annotation[]): Annotation | undefined {
  const result = annotations.filter(a => a.name.value == name)[0];
  return result;
}

function reduceType(type: Type, annotations: Annotation[] = []): TypeSignature {
  switch (type.getKind()) {
    case Kind.Named: {
      const t = type as Named;
      const name = t.name;
      if (isWidlType(name.value)) {
        return { type: name.value };
      } else {
        const link = getAnnotation('collection', annotations);
        if (name.value === 'link') {
          if (link) {
            const collection = link.arguments[0];
            return {
              type: 'link',
              collection: collection.value.getValue(),
            };
          } else {
            return {
              type: 'link',
            };
          }
        } else if (name.value === 'struct') {
          // TODO: convert this to core WIDL type once widl can represent it (see also: https://github.com/wapc/cli/issues/8)
          return { type: 'struct' };
        } else {
          return { type: 'ref', ref: `#/types/${name.value}` };
        }
      }
    }
    case Kind.MapType: {
      const t = type as MapType;
      return {
        type: 'map',
        key: reduceType(t.keyType, annotations),
        value: reduceType(t.valueType, annotations),
      };
    }
    case Kind.ListType: {
      const t = type as ListType;
      return {
        type: 'list',
        element: reduceType(t.type, annotations),
      };
    }
    case Kind.Optional: {
      const t = type as Optional;
      return {
        type: 'optional',
        option: reduceType(t.type, annotations),
      };
    }
  }
  throw new Error(`Unhandled type: ${type.getKind()}`);
}

function interpret(doc: Document): [ComponentSignature, RootType[], StructSignature?] {
  const types = doc.definitions.filter(isType);
  const input_def = findByName(types, /inputs/i);
  const output_def = findByName(types, /outputs/i);
  const config_def = findByName(types, /config/i);
  const namespace = doc.definitions.find(def => def.isKind(Kind.NamespaceDefinition));
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
    name: (namespace as NamespaceDefinition).name.value,
    inputs,
    outputs,
  };

  const typeSignatures = doc.definitions
    .filter(isSupportedType)
    .filter(t => !t.name.value.match(/inputs/i) && !t.name.value.match(/outputs/i) && !t.name.value.match(/config/i))
    .map(t => {
      switch (t.kind) {
        case TypeDefinition.name:
          return reduceTypeDefinition(t as TypeDefinition);
        case EnumDefinition.name:
          return reduceEnumDefinition(t as EnumDefinition);
        default:
          throw new Error(`Type ${t.kind} not yet handled`);
      }
    });
  return [component, typeSignatures, config];
}

function isType(def: Definition): def is TypeDefinition {
  return def.isKind(Kind.TypeDefinition);
}

function isSupportedType(def: Definition): def is TypeDefinition | EnumDefinition {
  return def.isKind(Kind.TypeDefinition) || def.isKind(Kind.EnumDefinition);
}

interface HasName extends AbstractNode {
  name: { value: string };
}

function findByName<T extends HasName>(defs: T[], name: string | RegExp): T | undefined {
  return defs.find(def => def.name.value.match(name));
}

function reduceTypeDefinition(def: TypeDefinition): StructSignature {
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

function reduceEnumDefinition(def: EnumDefinition): EnumSignature {
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
