---
title: CLI Trigger
weight: 3
file: data/examples/cli/wasm-calling-postgres.wick
description: A trigger whose events are initiated upon CLI execution and whose operations produce exit codes.
ref: clitrigger
---

Wick's {{<v1ref "clitrigger">}}CLI Trigger{{</v1ref>}} executes whenever an application is executed on the command line (which is almost always). It delegates CLI args and STDIO to an {{<v1ref "componentoperationexpression">}}operation{{</v1ref>}} which can optionally depend on arbitrary other components to provide dynamic CLI behavior.

### Example

*{{< metadata field = "description" >}}*

{{% app_config_header %}}

The CLI trigger is configured with an {{<v1ref "componentoperationexpression">}}operation{{</v1ref>}} that accepts `args: string[]` and `interactive: { stdin: bool, stdout: bool, stderr: bool }` as input.

This example uses two components, one that provides the CLI interface:

{{% value path = "import.name=CLI" %}}

And another that the first component requires, called MYDB. This is a SQL component that provides an interface whose implementation is backed by a SQL query vs compiled code. It's implementation isn't important for this example, but you can see it in the full configuration included at the end of this page.

The trigger configuration delegates operation to the `main` operation on our `CLI` component:

{{% value path = "triggers.kind=wick/trigger/cli@v1"  %}}

Our CLI trigger will execute when we run `wick run` on the command line.

{{% app_config_footer %}}