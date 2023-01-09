/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

import { Configuration } from "https://deno.land/x/apex_cli@v0.0.6/src/config.ts";
import * as apex from "https://deno.land/x/apex_core@v0.1.0/mod.ts";

interface Slug {
  value: string;
}

const importUrl = new URL(".", import.meta.url);

function urlify(relpath: string): string {
  const url = new URL(relpath, importUrl).toString();
  console.error(url);
  return url;
}

const docsMod = urlify("./docs.ts");

export default function (
  doc: apex.ast.Document,
  config: Configuration,
): Configuration {
  config.generates ||= {};
  config.config ||= {};

  const position = config.config.position_start as number | undefined;
  const vars = {
    position: position,
  };

  ["initializer", "transport", "router", "middleware", "filter", "action"]
    .forEach((type) => addComponentType(doc, config, vars, type));

  return config;
}

function addComponentType(
  doc: apex.ast.Document,
  config: Configuration,
  vars: Record<string, unknown>,
  type: string,
) {
  const actions = doc.definitions
    .filter((def) => def.isKind(apex.ast.Kind.TypeDefinition))
    .map((def) => def as apex.ast.TypeDefinition)
    .filter((t) => t.annotation(type) != undefined);

  actions.forEach((a) => {
    const position = vars.position as number | undefined;
    if (position) {
      vars.position = position+1;
    }
    const actionName = a.annotation(type)!.arguments[0].value
      .getValue() as string;
    let filename = actionName;
    const idx = filename.lastIndexOf("/");
    if (filename.startsWith("@") && idx != -1) {
      filename = filename.substring(idx + 1);
    }
    if (filename.endsWith("/v0") || filename.endsWith("/v1")) {
      filename = filename.substring(0, filename.length - 3);
    }
    a.annotation('slug', a => {
      filename = a.convert<Slug>().value;
    })
    config.generates[`./${filename}.mdx`] = {
      module: docsMod,
      visitorClass: "DocVisitor",
      config: {
        name: a.name.value,
        type: type,
        position: position,
      },
    };
  });
}
