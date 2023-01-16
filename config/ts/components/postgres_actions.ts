import { Component, Entity, ResourceRef, ValueExpr } from "../nanobus.ts";
import {
  ExecConfig,
  ExecMultiConfig,
  FindConfig,
  FindOneConfig,
  LoadConfig,
  Pagination,
  Preload,
  QueryConfig,
  Statement,
  Where,
} from "./actions_postgres.ts";
import * as postgres from "./actions_postgres.ts";

export interface Load {
  // Entity is the type entity to load.
  entity: Entity;
  // ID is the entity identifier expression.
  key: ValueExpr;
  // Preload lists the relationship to expand/load.
  preload?: Preload[];
  // NotFoundError is the error to return if the key is not found.
  notFoundError?: string;
}

export interface FindOne {
  // Entity is the type entity to find.
  entity: Entity;
  // Preload lists the relationship to expand/load.
  preload?: Preload[];
  // Where list the parts of the where clause.
  where?: Where[];
  // NotFoundError is the error to return if the key is not found.
  notFoundError: string;
}

export interface Find {
  // Entity is the type entity to find.
  entity: Entity;
  // Preload lists the relationship to expand/load.
  preload?: Preload[];
  // Where list the parts of the where clause.
  where?: Where[];
  // Pagination is the optional fields to wrap the results with.
  pagination?: Pagination;
  // Offset is the query offset.
  offset?: ValueExpr;
  // Limit is the query limit.
  limit?: ValueExpr;
}

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

  load(options: Load): Component<LoadConfig> {
    return postgres.Load({
      resource: this.db,
      ...options,
    });
  }

  findOne(options: FindOne): Component<FindOneConfig> {
    return postgres.FindOne({
      resource: this.db,
      ...options,
    });
  }

  find(options: Find): Component<FindConfig> {
    return postgres.Find({
      resource: this.db,
      ...options,
    });
  }
}
