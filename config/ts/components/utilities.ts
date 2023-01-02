import { Component, DataExpr, Handler } from "../nanobus.ts";
import {
  CallInterface,
  CallInterfaceConfig,
  CallProvider,
  CallProviderConfig,
  Log,
  LogConfig,
} from "./actions_core.ts";

export function log(format: string, ...args: unknown[]): Component<LogConfig> {
  return Log({
    format: format,
    args: args,
  });
}

export function callInterface(
  handler: Handler,
  input?: DataExpr,
): Component<CallInterfaceConfig> {
  return CallInterface({
    handler,
    input,
  });
}

export function callProvider(
  handler: Handler,
  input?: DataExpr,
): Component<CallProviderConfig> {
  return CallProvider({
    handler,
    input,
  });
}
