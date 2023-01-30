/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

import { Configuration } from "https://deno.land/x/apex_cli@v0.0.16/src/config.ts";
import * as apex from "./deps/core.ts";

interface Alias {
  import: string;
  type: string;
}
type Aliases = Record<string, Alias>;

const importUrl = new URL(".", import.meta.url);
function urlify(relpath: string): string {
  return new URL(relpath, importUrl).toString();
}

export default function (
  _doc: apex.ast.Document,
  config: Configuration,
): Configuration {
  const conf = config.config ||= {};
  const aliases = (conf.aliases ||= {}) as Aliases;
  const generates = config.generates ||= {};

  conf.logger = {
    import: "github.com/go-logr/logr",
    interface: "logr.Logger",
  };
  conf.writeTypeInfo = false;
  conf.mapstructureTag = true;

  aliases["Duration"] = {
    import: "time",
    type: "time.Duration",
  };
  aliases["TextExpr"] = {
    import: "github.com/nanobus/nanobus/pkg/expr",
    type: "*expr.Text",
  };
  aliases["ValueExpr"] = {
    import: "github.com/nanobus/nanobus/pkg/expr",
    type: "*expr.ValueExpr",
  };
  aliases["DataExpr"] = {
    import: "github.com/nanobus/nanobus/pkg/expr",
    type: "*expr.DataExpr",
  };
  aliases["Handler"] = {
    import: "github.com/nanobus/nanobus/pkg/handler",
    type: "handler.Handler",
  };
  aliases["Step"] = {
    import: "github.com/nanobus/nanobus/pkg/runtime",
    type: "runtime.Step",
  };
  aliases["Component"] = {
    import: "github.com/nanobus/nanobus/pkg/runtime",
    type: "runtime.Component",
  };
  aliases["FilePath"] = {
    import: "github.com/nanobus/nanobus/pkg/runtime",
    type: "runtime.FilePath",
  };
  aliases["Entity"] = {
    import: "github.com/nanobus/nanobus/pkg/entity",
    type: "entity.Entity",
  };
  aliases["ResourceRef"] = {
    import: "github.com/nanobus/nanobus/pkg/resource",
    type: "resource.Ref",
  };

  generates["generated.go"] = {
    module: urlify("./components.ts"),
    visitorClass: "ComponentsVisitor",
  };

  return config;
}
