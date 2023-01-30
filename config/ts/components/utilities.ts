import { Component, DataExpr, Handler } from "../nanobus.ts";
import {
  CallInterface,
  CallInterfaceConfig,
  CallProvider,
  CallProviderConfig,
  Expr,
  ExprConfig,
  Log,
  LogConfig,
} from "./actions_core.ts";
import { AddRoute, RouterV1, RouterV1Config } from "./transport_router.ts";

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

/// Expr utilities

export function value(expr: string): Component<ExprConfig> {
  return Expr({
    value: expr,
  });
}

export function transform(expr: string): Component<ExprConfig> {
  return Expr({
    data: expr,
  });
}

/// RouterV1 utilities

export function router(...routes: AddRoute[]): Component<RouterV1Config> {
  return RouterV1({
    routes: routes,
  })
}

export function GET(
  uri: string,
  handler: Handler,
  raw?: boolean,
  encoding?: string,
): AddRoute {
  return { method: "GET", uri, handler, raw, encoding };
}

export function POST(
  uri: string,
  handler: Handler,
  raw?: boolean,
  encoding?: string,
): AddRoute {
  return { method: "POST", uri, handler, raw, encoding };
}

export function PUT(
  uri: string,
  handler: Handler,
  raw?: boolean,
  encoding?: string,
): AddRoute {
  return { method: "PUT", uri, handler, raw, encoding };
}

export function PATCH(
  uri: string,
  handler: Handler,
  raw?: boolean,
  encoding?: string,
): AddRoute {
  return { method: "PATCH", uri, handler, raw, encoding };
}

export function DELETE(
  uri: string,
  handler: Handler,
  raw?: boolean,
  encoding?: string,
): AddRoute {
  return { method: "DELETE", uri, handler, raw, encoding };
}

export function OPTIONS(
  uri: string,
  handler: Handler,
  raw?: boolean,
  encoding?: string,
): AddRoute {
  return { method: "OPTIONS", uri, handler, raw, encoding };
}

export function HEAD(
  uri: string,
  handler: Handler,
  raw?: boolean,
  encoding?: string,
): AddRoute {
  return { method: "HEAD", uri, handler, raw, encoding };
}
