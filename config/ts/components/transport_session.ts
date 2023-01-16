// Code generated by NanoBus codegen utilities. DO NOT EDIT.

// deno-lint-ignore-file no-explicit-any no-unused-vars ban-unused-ignore
import {
  CodecRef,
  Component,
  DataExpr,
  Handler,
  Entity,
  ResourceRef,
  Step,
  ValueExpr
} from "../nanobus.ts";

export interface SessionV1Config {
  handler: Handler;
}

export function SessionV1(config: SessionV1Config): Component<SessionV1Config> {
  return {
    uses: "nanobus.filter.session/v1",
    with: config
  };
}
