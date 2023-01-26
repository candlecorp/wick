// deno-lint-ignore-file require-await
import {
  FSStructure,
  Template,
} from "https://deno.land/x/apex_cli@v0.0.16/src/config.ts";

const template: Template = {
  info: {
    name: "@nanobus/tinygo",
    description: "NanoBus TinyGo Iota",
    variables: [
      {
        name: "module",
        description: "The module name",
        type: "input",
        prompt:
          "Please enter the module name (e.g. github.com/myorg/myservice)",
        default: "github.com/myorg/myservice",
      },
      {
        name: "package",
        description: "The main package name",
        type: "input",
        prompt: "Please enter the main package name (e.g. myservice)",
        default: "myservice",
      },
    ],
  },

  async process(_vars): Promise<FSStructure> {
    const version = getVersion(import.meta.url);
    const pluginVer = version ? `@${version}` : '';

    return {
      variables: {
        plugin: `https://deno.land/x/nanobusconfig${pluginVer}/plugin.ts`,
      },
      files: [
        ".vscode/extensions.json",
        ".vscode/settings.json",
        ".vscode/tasks.json",
        "apex.axdl",
      ],
      templates: {
        "tmpl": [
          "apex.yaml.tmpl",
          "bus.ts.tmpl",
          "go.mod.tmpl",
        ],
      },
    };
  },
};

function getVersion(str: string): string | undefined {
  const regexVersion = /@(v[0-9][^\/]*)\//gm;

  // Get version
  let m;
  let version: string | undefined;
  if ((m = regexVersion.exec(str)) !== null) {
    m.forEach((match, groupIndex) => {
      //tmpl.version = match;
      if (groupIndex == 1) {
        version = match;
      }
    });
  }
  return version;
}

export default template;
