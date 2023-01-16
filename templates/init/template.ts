// deno-lint-ignore-file require-await
import {
  FSStructure,
  Template,
} from "https://deno.land/x/apex_cli@v0.0.15/src/config.ts";

const template: Template = {
  info: {
    name: "@nanobus/init",
    description: "NanoBus barebones Iota",
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
        "apex.yaml",
      ],
      templates: {
        "tmpl": [
          "bus.yaml.tmpl",
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
