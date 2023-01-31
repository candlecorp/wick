import { Configuration } from "./deps/cli.ts";
import * as apex from "./deps/core.ts";

const importUrl = new URL(".", import.meta.url);

function urlify(relpath: string): string {
  return new URL(relpath, importUrl).toString();
}

function taskName(taskExpr: string): string {
  const idx = taskExpr.indexOf(">");
  if (idx != -1) {
    return taskExpr.substring(idx).trim();
  }
  return taskExpr.trim();
}

export default function (
  _doc: apex.ast.Document,
  config: Configuration,
): Configuration {
  config.config ||= {};
  config.generates ||= {};
  const generates = config.generates || [];
  config.generates = generates;

  const mode = config.config.mode as string || "export";
  const file = config.config.iotaTypeScriptPath as string || "iota.ts";
  generates[file] = {
    module: urlify("./iota_visitor.ts"),
  };

  if (mode == "export") {
    const tasks = config.tasks ||= {};
    const names = new Set<string>(Object.keys(tasks).map((k) => taskName(k)));
    const defaultTasks: Record<string, string[]> = {
      start: [
        "deno run --allow-write bus.ts bus.yaml",
        "nanobus run --debug --developer-mode",
      ],
    };
    for (const key of Object.keys(defaultTasks)) {
      if (!names.has(taskName(key))) {
        tasks[key] = defaultTasks[key];
      }
    }
  }

  return config;
}
