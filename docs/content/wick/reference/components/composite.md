---
title: Composite
weight: 2
file: data/examples/components/echo.wick
ref: compositecomponentconfiguration
description: A component made up of other components and whose operations are defined as a data flow between other operations.
---

***This Documentation is a WIP***


## Example

*{{< metadata field = "description" >}}*

{{% component_config_header %}}

This example defines {{% oplist %}}. There is no real logic in `composite` operations. They define the flow of data from the input, to other operations, and finally to the output.

The `inputs` and `outputs` are frequently omitted in composite operations. Their types are inferred from the inputs and outputs they connect to.

{{% value path = "component.operations" highlight = true %}}

{{% component_config_footer %}}