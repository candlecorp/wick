import {
  Template,
} from "https://deno.land/x/apex_cli@v0.0.16/src/config.ts";

const template: Template = {
  info: {
    name: "@nanobus",
    description: "NanoBus template suite",
  },

  templates: [
    "./init/template.ts",
    "./init-ts/template.ts",
    "./tinygo/template.ts",
  ],
};

export default template;
