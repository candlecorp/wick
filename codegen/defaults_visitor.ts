/*
Copyright 2022 The Apex Authors.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

import { Context, Writer, BaseVisitor } from './deps/model.ts';
import { Kind } from './deps/ast.ts';
import { formatComment, isNamed } from './deps/utils.ts';
import { expandType, fieldName, translateAlias } from './deps/apex.ts';

interface Conversion {
  from: string;
  to: string;
  do: string;
}

export default class DefaultsVisitor extends BaseVisitor {
  private writeTypeInfo: boolean;

  constructor(writer: Writer, writeTypeInfo: boolean = false) {
    super(writer);
    this.writeTypeInfo = writeTypeInfo;
  }

  visitContextBefore(context: Context): void {
    this.write(`package ${context.config.package}\n`);
    const toImport: string[] = context.config.imports || [];

    const imports = toImport.map((i) => `"${i}"`).join('\n');
    this.write(`
    import (
      ${imports}
    )\n`);
  }

  visitTypeBefore(context: Context): void {
    const { type } = context;
    super.triggerTypeBefore(context);
    this.write(
      formatComment(
        `// `,
        `Returns a ${type.name} instance with default fields populated`
      )
    );
    this.write(`
    func Default${type.name}() ${type.name} {
      obj := ${type.name} {}
    `);
  }

  visitTypeField(context: Context): void {
    const { field } = context;
    const conversions: Conversion[] = context.config.conversions || [];

    if (field.default) {
      let literal = field.default.getValue();
      const fromKind = field.default.getKind();
      const toKind = isNamed(field.type) ? field.type.name : field.type.kind;
      if (fromKind === Kind.StringValue) {
        literal = `"${literal}"`;
      }
      const conversion = conversions.find(
        (def) => fromKind == def.from && toKind == def.to
      );
      if (conversion) {
        const fromType = expandLiteralType(fromKind);
        const toType = expandType(
          field.type!,
          context.config.package,
          true,
          translateAlias(context)
        );
        this.write(
          `obj.${fieldName(
            field,
            field.name
          )} = (func(value ${fromType}) ${toType} { ${
            conversion.do
          } })(${literal})\n`
        );
      } else {
        this.write(`obj.${fieldName(field, field.name)} = ${literal}\n`);
      }
    }
    super.triggerTypeField(context);
  }

  visitTypeAfter(context: Context): void {
    const { type } = context;
    this.write(`
return obj\n}

func (h *${type.name}) UnmarshalYAML(unmarshal func(interface{}) error) error {
	type alias ${type.name}
	raw := alias(Default${type.name}())
	if err := unmarshal(&raw); err != nil {
		return err
	}
	*h = ${type.name}(raw)
	return nil
}
`);
    super.triggerTypeAfter(context);
  }
}

function expandLiteralType(kind: Kind): string {
  switch (kind) {
    case Kind.StringValue:
      return 'string';
    case Kind.IntValue:
      return 'int';
    case Kind.FloatValue:
      return 'float64';
    case Kind.BooleanValue:
      return 'bool';
  }
  throw new Error(`unhandled literal type: ${kind}`);
}
