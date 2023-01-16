/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

import {
  Alias,
  Annotated,
  Context,
  Kind,
  Named,
  Optional,
  Primitive,
  PrimitiveName,
  Writer,
} from "./deps/model.ts";
import {
  expandType,
  fieldName,
  getImporter,
  GoVisitor,
  InterfacesVisitor,
  StructVisitor,
} from "./deps/apex.ts";
import { formatComment, typeName } from "./deps/utils.ts";

interface Validate {
  value: string;
}

interface Component {
  value: string;
}

const IMPORTS = {
  initialize: "github.com/nanobus/nanobus/pkg/initialize",
  transport: "github.com/nanobus/nanobus/pkg/transport",
  router: "github.com/nanobus/nanobus/pkg/transport/http/router",
  middleware: "github.com/nanobus/nanobus/pkg/transport/http/middleware",
  filter: "github.com/nanobus/nanobus/pkg/transport/filter",
  actions: "github.com/nanobus/nanobus/pkg/actions",
};

const requiredAliases = [
  "ValueExpr",
  "DataExpr",
  "ResourceRef",
  "Handler",
  "Entity",
];

class ConfigStructVisitor extends StructVisitor {
  structTags(context: Context): string {
    const { field } = context;
    let tags = "";
    let validate = "";

    field.annotation("validate", (a) => {
      const v = a.convert<Validate>();
      if (validate.length > 0) {
        validate += ",";
      }
      validate += v.value;
    });
    if (validate.length == 0) {
      let t = field.type;
      if (t.kind == Kind.Optional) {
        t = (t as Optional).type;
      } else if (
        (t.kind == Kind.Primitive &&
          (t as Primitive).name == PrimitiveName.String)
      ) {
        validate += `required`;
      } else if (t.kind == Kind.Alias) {
        const a = t as Alias;
        if (requiredAliases.indexOf(a.name) != -1) {
          validate += `required`;
        }
      }

      if (t.kind == Kind.Map || t.kind == Kind.List) {
        if (validate.length > 0) {
          validate += ",";
        }
        validate += "dive";
      }
    }
    if (validate.length > 0) {
      tags += ` validate:"${validate}"`;
    }
    return tags;
  }
}

interface UnionKey {
  value: string;
}

class ConfigUnionVisitor extends GoVisitor {
  visitUnion(context: Context): void {
    const tick = "`";
    const { union } = context;
    this.write(formatComment("// ", union.description));
    this.write(`type ${union.name} struct {\n`);
    union.types.forEach((t) => {
      let tname = typeName(t);
      const annotated = t as Annotated;
      if (annotated.annotation) {
        annotated.annotation("unionKey", (a) => {
          tname = a.convert<UnionKey>().value;
        });
      }

      const without = union.types.map((t) => {
        const annotated = t as Annotated;
        let tname = typeName(t);
        if (annotated.annotation) {
          annotated.annotation("unionKey", (a) => {
            tname = a.convert<UnionKey>().value;
          });
        }
        return tname;
      }).filter((t) => t != tname).map((t) =>
        fieldName(undefined as unknown as Annotated, t)
      ).join(",");

      const expandedName = expandType(t);
      this.write(
        `${
          fieldName(
            undefined as unknown as Annotated,
            tname,
          )
        } *${expandedName} ${tick}json:"${tname},omitempty" yaml:"${tname},omitempty" msgpack:"${tname},omitempty" validate:"required_without=${without}`,
      );
      this.triggerCallbacks(context, "UnionStructTags");
      this.write(`"${tick}\n`);
    });
    this.write(`}\n\n`);
  }
}

export class ComponentsVisitor extends InterfacesVisitor {
  constructor(writer: Writer) {
    super(writer);
    this.structVisitor = () => new ConfigStructVisitor(writer);
    this.unionVisitor = () => new ConfigUnionVisitor(writer);
  }

  visitAlias(context: Context): void {
    super.visitAlias(context);
    const { alias } = context;
    this.doLoader(context, alias, alias);
  }

  visitTypeAfter(context: Context): void {
    const { type } = context;
    this.doLoader(context, type, type);
  }

  doLoader(context: Context, named: Named, annotated: Annotated): void {
    const $ = getImporter(context, IMPORTS);
    const name = named.name.replaceAll(/(Config|Configuration|Settings)$/g, "");

    annotated.annotation("initializer", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, ${$.initialize}.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });

    annotated.annotation("transport", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, ${$.transport}.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });

    annotated.annotation("router", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, ${$.router}.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });

    annotated.annotation("middleware", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, ${$.middleware}.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });

    annotated.annotation("filter", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, ${$.filter}.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });

    annotated.annotation("action", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, ${$.actions}.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });
  }
}
