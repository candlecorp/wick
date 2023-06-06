---
title: Time Trigger
weight: 1
file: data/examples/time/time.wick
description: A trigger whose events are time-based and whose operations’ output is logged.
ref: timetrigger
---

Wick's {{<v1ref "timetrigger">}}Time Trigger{{</v1ref>}} takes a cron-like configuration and calls a configured {{<v1ref "componentoperationexpression">}}operation{{</v1ref>}} on a schedule.

Any number of {{<v1ref "timetrigger">}}Time Triggers{{</v1ref>}} can be added to an application.

### Example

*{{< metadata field = "description" >}}*

{{% app_config_header %}}

The time trigger takes an {{<v1ref "componentoperationexpression">}}operation{{</v1ref>}}, an optional list of {{<v1ref "operationinput">}}inputs{{</v1ref>}} to pass to the `payload` on execution, and a {{<v1ref "schedule">}}schedule{{</v1ref>}}.

{{% value path = "triggers.kind=wick/trigger/time@v1"  %}}

A {{<v1ref "schedule">}}schedule{{</v1ref>}} takes a required cron string and an optional `repeat` parameter which acts as a maximum number of executions. If you need help with cron syntax, visit [cron.help](https://cron.help).

{{% value path = "triggers.kind=wick/trigger/time@v1.schedule" context = true %}}

{{% app_config_footer %}}