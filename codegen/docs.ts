/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

import {
  AnyType,
  BaseVisitor,
  Context,
  Enum,
  Kind,
  List,
  Map as MapType,
  Named,
  Optional,
  Type,
} from "./deps/core.ts";

const ticks = "```";

interface StringValue {
  value: string;
}

interface Tags {
  value: string[];
}

interface Examples {
  value: Example[];
}

interface Example {
  title: string;
  formats: Record<string, string>;
}

export class DocVisitor extends BaseVisitor {
  visitContextBefore(context: Context): void {
    const typeName = context.config.name as string;
    const compType = context.config.type as string;
    const position = context.config.position as number | undefined;
    const pkg = context.config.pkg as string;
    const type = context.namespace.types[typeName];
    const componentName = type.annotation(compType)?.arguments[0].value
      .getValue();
    let title = componentName;
    const idx = title.lastIndexOf("/");
    if (title.startsWith("@") && idx != -1) {
      title = title.substring(idx + 1);
    }
    if (title.endsWith("/v0") || title.endsWith("/v1")) {
      title = title.substring(0, title.length - 3);
    }
    type.annotation("title", (a) => title = a.convert<StringValue>().value);

    let filename = componentName;
    type.annotation("slug", (a) => filename = a.convert<StringValue>().value);
    type.annotation(
      "filename",
      (a) => filename = a.convert<StringValue>().value,
    );

    const refTypes = new Set<AnyType>();
    type.fields.forEach((f) => {
      getTypeReferences(f.type, refTypes);
    });

    this.write(`---
title: ${title}\n`);
    if (position) {
      this.write(`sidebar_position: ${position}\n`);
    }
    this.write(`---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';
\n`);

    this.write(`# ${componentName}

<div class="attributes">

### Description

<p>\n`);
    type.annotation("tags", (a) => {
      a.convert<Tags>().value.forEach((tag) =>
        this.write(`  <span className="badge badgeDarkBlue">${tag}</span>\n`)
      );
    });
    this.write(
      `  <a href="https://github.com/nanobus/nanobus/blob/main/pkg/${pkg}/${filename}.go" target="_blank" rel="noopener noreferrer">Source code <svg width="13.5" height="13.5" aria-hidden="true" viewBox="0 0 24 24" class="iconExternalLink_node_modules-@docusaurus-theme-classic-lib-theme-Icon-ExternalLink-styles-module"><path fill="currentColor" d="M21 13v10h-21v-19h12v2h-10v15h17v-8h2zm3-12h-10.988l4.035 4-6.977 7.07 2.828 2.828 6.977-7.07 4.125 4.172v-11z"></path></svg></a>
</p>

</div>

${type.description || ""}\n\n`,
    );

    // List configiration options
    if (type.fields && type.fields.length > 0) {
      this.write(`## Options\n\n`);
      this.renderType(type);
    }

    refTypes.forEach((anyType) => {
      switch (anyType.kind) {
        case Kind.Type: {
          const t = anyType as Type;
          this.write(`## type ${t.name}\n\n`);
          this.renderType(t);
          break;
        }
        case Kind.Enum: {
          const e = anyType as Enum;
          this.write(`## enum ${e.name}\n\n`);
          this.renderEnum(e);
          break;
        }
      }
    });

    // List examples
    type.annotation("examples", (a) => {
      this.write(`## Examples\n\n`);

      const examples = a.convert<Examples>();

      examples.value.forEach((example) => {
        this.write(`### ${example.title}

<Tabs
  values={[\n`);
        Object.keys(example.formats)
          .forEach((key) =>
            this.write(
              `    {label: '${key}', value: '${key.toLowerCase()}'},\n`,
            )
          );
        this.write(`  ]}>\n`);
        for (const key of Object.keys(example.formats)) {
          const content = example.formats[key];
          this.write(`  <TabItem value="${key.toLowerCase()}">

${ticks}${key.toLowerCase()}
${content}
${ticks}

  </TabItem>\n`);
        }
        this.write(`</Tabs>\n`);
      });
    });
  }

  renderType(type: Type) {
    this.write(`<div class="attributes">\n\n`);

    type.fields.forEach((f) => {
      this.write(`### ${f.name}\n\n`);

      let ft = f.type;
      let required = true;
      if (ft.kind == Kind.Optional) {
        ft = (ft as Optional).type;
        required = false;
      }
      const typeName = expandType(ft, false);

      this.write(`<div class="attribute">
  <header>
    <span className="badge badgeDarkBlue">${f.name}${
        required ? ` <span className="required">*</span>` : ``
      }
      <a class="hash-link" href="#${f.name.toLowerCase()}" title="Direct link to ${f.name}">​</a>
    </span>
    <code>${typeName}</code>
  </header>

${f.description || ""}

</div>\n\n`);
    });

    this.write(`</div>\n\n`);
  }

  renderEnum(en: Enum) {
    this.write(`<div class="attributes">\n\n`);

    en.values.forEach((v) => {
      const value = v.display || v.name;
      this.write(`### ${value}\n\n`);

      this.write(`<div class="attribute">
  <header>
    <span className="badge badgeDarkBlue">${value}
      <a class="hash-link" href="#${value.toLowerCase()}" title="Direct link to ${value}">​</a>
    </span>
  </header>

${v.description || ""}

</div>\n\n`);
    });

    this.write(`</div>\n\n`);
  }
}

const translations = new Map<string, string>([
  ["ValueExpr", "string (Value expression)"],
  ["DataExpr", "string (Data expression)"],
]);

function expandType(type: AnyType, useOptional: boolean): string {
  switch (type.kind) {
    case Kind.Type: {
      const t = type as Type;
      return `<a href="#type-${t.name.toLowerCase()}">${
        escapeHTML(t.name)
      }</a>`;
    }
    case Kind.Primitive:
    case Kind.Alias:
    case Kind.Enum:
    case Kind.Union: {
      const namedValue = (type as Named).name;
      const translation = translations.get(namedValue);
      if (translation != undefined) {
        return escapeHTML(translation!);
      }
      return escapeHTML(namedValue);
    }
    case Kind.Map:
      return `Map&lt;${expandType((type as MapType).keyType, true)},${
        expandType(
          (type as MapType).valueType,
          true,
        )
      }&gt;`;
    case Kind.List:
      return `${expandType((type as List).type, true)}[]`;
    case Kind.Optional: {
      const expanded = expandType((type as Optional).type, true);
      if (useOptional) {
        return `${expanded}?`;
      }
      return expanded;
    }
    default:
      return "unknown";
  }
}

function getTypeReferences(type: AnyType, types: Set<AnyType>) {
  if (types.has(type)) {
    return;
  }
  switch (type.kind) {
    case Kind.Enum:
      types.add(type);
      break;
    case Kind.Type: {
      const t = type as Type;
      types.add(type);
      t.fields.forEach((f) => getTypeReferences(f.type, types));
      break;
    }
    case Kind.List:
      getTypeReferences((type as List).type, types);
      break;
    case Kind.Map: {
      const m = type as MapType;
      getTypeReferences(m.keyType, types);
      getTypeReferences(m.valueType, types);
      break;
    }
    case Kind.Optional:
      getTypeReferences((type as Optional).type, types);
      break;
  }
}

function escapeHTML(value: string): string {
  return value.replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}
