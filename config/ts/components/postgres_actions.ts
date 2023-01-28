import { Component, Entity, ResourceRef, ValueExpr } from "../nanobus.ts";
import {
  ExecConfig,
  ExecMultiConfig,
  FindConfig,
  FindOneConfig,
  LoadConfig,
  QueryConfig,
  Statement,
} from "./actions_postgres.ts";
import * as postgres from "./actions_postgres.ts";

export class PostgresActions {
  db: ResourceRef;

  constructor(db: ResourceRef) {
    this.db = db;
  }

  queryOne(sql: string, ...args: unknown[]): Component<QueryConfig> {
    return postgres.Query({
      resource: this.db,
      single: true,
      sql: sql,
      args: args,
    });
  }

  query(sql: string, ...args: unknown[]): Component<QueryConfig> {
    return postgres.Query({
      resource: this.db,
      sql: sql,
      args: args,
    });
  }

  exec(sql: string, ...args: unknown[]): Component<ExecConfig> {
    return postgres.Exec({
      resource: this.db,
      sql: sql,
      args: args,
    });
  }

  execMulti(...statements: Statement[]): Component<ExecMultiConfig> {
    return postgres.ExecMulti({
      resource: this.db,
      statements,
    });
  }

  load(
    entity: Entity,
    key: ValueExpr,
    options: Omit<LoadConfig, "resource" | "entity" | "key"> = {},
  ): Component<LoadConfig> {
    return postgres.Load({
      resource: this.db,
      entity,
      key,
      ...options,
    });
  }

  findOne(
    entity: Entity,
    options: Omit<FindOneConfig, "resource" | "entity"> = {},
  ): Component<FindOneConfig> {
    return postgres.FindOne({
      resource: this.db,
      entity,
      ...options,
    });
  }

  find(
    entity: Entity,
    options: Omit<FindConfig, "resource" | "entity"> = {},
  ): Component<FindConfig> {
    return postgres.Find({
      resource: this.db,
      entity,
      ...options,
    });
  }
}
