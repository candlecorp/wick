---
title: Short form
weight: 1
---

# ComponentDefinition

Inline `ComponentDefinition`'s can be substituted with a component ID used in the import statement.


# ComponentOperationExpression

Any `ComponentOperationExpression` value can be substituted with the component ID and the operation name in the format `component_id::operation_name`.

For example:

```yaml
operation:
  component: component_id
  name: operation_name
```

Can be shortened to

```yaml
operation: component_id::operation_name
```
