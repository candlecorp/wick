/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

import {
  Annotated,
  Context,
  Kind,
  Named,
  Primitive,
  PrimitiveName,
  Writer,
} from "./deps/core.ts";
import {
  ImportsVisitor,
  InterfacesVisitor,
  StructVisitor,
} from "./deps/apex.ts";

interface Validate {
  value: string;
}

interface Component {
  value: string;
}

class ComponentImportsVisitor extends ImportsVisitor {
  visitAlias(context: Context): void {
    super.visitAlias(context);
    const { alias } = context;
    this.doImports(alias);
  }

  visitType(context: Context): void {
    const { type } = context;
    this.doImports(type);
  }

  doImports(annotated: Annotated): void {
    annotated.annotation("initializer", (_a) => {
      this.addType("initialize", {
        type: "initialize",
        import: "github.com/nanobus/nanobus/pkg/initialize",
      });
    });
    annotated.annotation("transport", (_a) => {
      this.addType("transport", {
        type: "transport",
        import: "github.com/nanobus/nanobus/pkg/transport",
      });
    });
    annotated.annotation("router", (_a) => {
      this.addType("router", {
        type: "router",
        import: "github.com/nanobus/nanobus/pkg/transport/http/router",
      });
    });
    annotated.annotation("middleware", (_a) => {
      this.addType("middleware", {
        type: "middleware",
        import: "github.com/nanobus/nanobus/pkg/transport/http/middleware",
      });
    });
    annotated.annotation("filter", (_a) => {
      this.addType("filter", {
        type: "filter",
        import: "github.com/nanobus/nanobus/pkg/transport/filter",
      });
    });
    annotated.annotation("action", (_a) => {
      this.addType("action", {
        type: "actions",
        import: "github.com/nanobus/nanobus/pkg/actions",
      });
    });
  }
}

class ConfigStructVisitor extends StructVisitor {
  structTags(context: Context): string {
    const { field } = context;
    let tags = "";
    let validate = "";

    field.annotation("validate", (a) => {
      const v = a.convert<Validate>();
      if (validate.length > 0) {
        validate += " ";
      }
      validate += v.value;
    });
    if (
      validate.length == 0 &&
      field.type.kind != Kind.Optional &&
      !(
        field.type.kind == Kind.Primitive &&
        (field.type as Primitive).name == PrimitiveName.Bool
      )
    ) {
      validate += `required`;
    }
    if (validate.length > 0) {
      tags += ` validate:"${validate}"`;
    }
    return tags;
  }
}

export class ComponentsVisitor extends InterfacesVisitor {
  constructor(writer: Writer) {
    super(writer);
    this.importsVisitor = () => new ComponentImportsVisitor(writer);
    this.structVisitor = () => new ConfigStructVisitor(writer);
  }

  visitAlias(context: Context): void {
    super.visitAlias(context);
    const { alias } = context;
    this.doLoader(alias, alias);
  }

  visitTypeAfter(context: Context): void {
    const { type } = context;
    this.doLoader(type, type);
  }

  doLoader(named: Named, annotated: Annotated): void {
    const name = named.name.replaceAll(/(Config|Configuration|Settings)$/g, "");

    annotated.annotation("initializer", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, initialize.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });

    annotated.annotation("transport", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, transport.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });

    annotated.annotation("router", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, router.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });

    annotated.annotation("middleware", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, middleware.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });

    annotated.annotation("filter", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, filter.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });

    annotated.annotation("action", (a) => {
      const component = a.convert<Component>();
      this.write(`
func ${name}() (string, actions.Loader) {
  return "${component.value}", ${name}Loader
}\n\n`);
    });
  }
}
