/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

import * as YAML from "https://deno.land/std@0.167.0/encoding/yaml.ts";
import { Duration as Dur } from "https://deno.land/x/durationjs@v4.0.0/mod.ts";

export type ResourceRef = string & { __desc: "Resource" };
export type Duration = string & { __desc: "Duration" };
export type ValueExpr = string;
export type DataExpr = string;
export type CodecRef = string & { __desc: "Codec" };
export type Timeout = Duration;
export type Handler = string & { __desc: "Handler" };

type Operations = {
  [operation: string]: Step[];
};

type Pipelines = {
  [operation: string]: Pipeline;
};

interface Pipeline {
  steps: Step[];
}

type Handlers<Type> = {
  [Property in keyof Type]: Handler;
};

type Timeouts = {
  [name: string]: Duration;
};

type TimeoutRefs<Type> = {
  [Property in keyof Type]: TimeoutRef;
};

type Retries = {
  [name: string]: Backoff;
};

type RetryRefs<Type> = {
  [Property in keyof Type]: RetryRef;
};

type CircuitBreakers = {
  [name: string]: CircuitBreaker;
};

type CircuitBreakerRefs<Type> = {
  [Property in keyof Type]: CircuitBreakerRef;
};

function isInteger(str: string) {
  if (typeof str !== "string") {
    return false;
  }
  const num = Number(str);
  return Number.isInteger(num);
}

export function duration(value: string): Duration {
  value = value.trim();
  if (isInteger(value)) {
    value += "ms";
  }
  const d = Dur.fromString(value);
  if (d.array.length == 0) {
    throw new Error(`bad duration ${value}`);
  }
  const s = d.array
    .filter((x) => x.value > 0)
    .map((x) => `${x.value}${x.type}`)
    .join(" ");
  if (s == "") {
    throw new Error(`bad duration ${value}`);
  }
  return s as Duration;
}

export const codecs: { [name: string]: CodecRef } = {
  JSON: "json" as CodecRef,
  MsgPack: "msgpack" as CodecRef,
  CloudEventsJSON: "cloudevents+json" as CodecRef,
};

export interface IncludeOptions {
  resourceLinks?: { [key: string]: ResourceRef };
}

export interface Iota<T> {
  $ref: string;
  interfaces: T;
}

// For YAML serialization
interface Ref extends IncludeOptions {
  ref: string;
}

export function env(key: string): string {
  return "${env:" + key + "}";
}

export interface ErrorTemplate {
  type: string;
  code: string;
  title: string;
  message: string;
}

export interface Package {
  registry: string;
  org: string;
  add?: string[];
}

interface AppConfig {
  readonly id: string;
  readonly version: string;
  spec?: string;
  main?: string;
  package?: Package;
  readonly resources: ResourceRef[];
  readonly includes: { [key: string]: Ref };
  readonly resiliency: Resiliency;
  readonly initializers: { [key: string]: Component<unknown> };
  readonly transports: { [key: string]: Component<unknown> };
  readonly filters: Component<unknown>[];
  readonly preauth: { [key: string]: Pipelines };
  readonly authorization: { [key: string]: Authorizations };
  readonly postauth: { [key: string]: Pipelines };
  readonly interfaces: { [key: string]: Pipelines };
  readonly providers: { [key: string]: Pipelines };
  readonly errors: { [key: string]: ErrorTemplate };
}

export interface Module {
  initialize(app: Application): void;
}

export class Application {
  readonly config: AppConfig;

  constructor(id: string, version: string) {
    this.config = {
      id,
      version,
      spec: undefined,
      main: undefined,
      package: undefined,
      resources: [],
      includes: {},
      resiliency: {
        timeouts: {},
        retries: {},
        circuitBreakers: {},
      },
      initializers: {},
      transports: {},
      filters: [],
      preauth: {},
      authorization: {},
      postauth: {},
      interfaces: {},
      providers: {},
      errors: {},
    };
  }

  spec(spec: string): Application {
    this.config.spec = spec;
    return this;
  }

  main(main: string): Application {
    this.config.main = main;
    return this;
  }

  package(pkg: Package): Application {
    this.config.package = pkg;
    return this;
  }

  use(...modules: Module[]): Application {
    modules.forEach((module) => module.initialize(this));
    return this;
  }

  resource(name: string): ResourceRef {
    const ref: ResourceRef = name as ResourceRef;
    this.config.resources.push(ref);
    return ref;
  }

  timeouts<T extends Timeouts>(arg: T): TimeoutRefs<T> {
    const ret: { [name: string]: TimeoutRef } = {};
    for (const key of Object.keys(arg)) {
      this.config.resiliency.timeouts[key] = arg[key];
      ret[key] = key as TimeoutRef;
    }
    return ret as TimeoutRefs<T>;
  }

  retries<T extends Retries>(arg: T): RetryRefs<T> {
    const ret: { [name: string]: RetryRef } = {};
    for (const key of Object.keys(arg)) {
      const value = arg[key];
      this.config.resiliency.retries[key] = value;
      ret[key] = key as RetryRef;
    }
    return ret as RetryRefs<T>;
  }

  circuitBreakers<T extends CircuitBreakers>(arg: T): CircuitBreakerRefs<T> {
    const ret: { [name: string]: CircuitBreakerRef } = {};
    for (const key of Object.keys(arg)) {
      this.config.resiliency.circuitBreakers[key] = arg[key];
      ret[key] = key as CircuitBreakerRef;
    }
    return ret as CircuitBreakerRefs<T>;
  }

  constantBackoff(name: string, dur: string): RetryRef {
    this.config.resiliency.retries[name] = {
      constant: {
        duration: duration(dur),
      },
    };
    return name as RetryRef;
  }

  exponentialBackoff(name: string, config: ExponentialBackoff): RetryRef {
    this.config.resiliency.retries[name] = {
      exponential: config,
    };
    return name as RetryRef;
  }

  circuitBreaker(name: string, config: CircuitBreaker): CircuitBreakerRef {
    this.config.resiliency.circuitBreakers[name] = config;
    return name as CircuitBreakerRef;
  }

  include<T>(
    instanceId: string,
    iota: Iota<T>,
    options: IncludeOptions = {},
  ): T {
    this.config.includes[instanceId] = {
      ref: iota.$ref,
      ...options,
    };
    return iota.interfaces;
  }

  initializer(name: string, comp: Component<unknown>): Application {
    this.config.initializers[name] = comp;
    return this;
  }

  transport(name: string, comp: Component<unknown>): Application {
    this.config.transports[name] = comp;
    return this;
  }

  authorizations(rules: Record<Handler, Authorization>) {
    for (const handler of Object.keys(rules)) {
      const rule = rules[handler as Handler];
      const [iface, operation] = handler.split("::");
      let exsting = this.config.authorization[iface];
      if (!exsting) {
        exsting = {};
        this.config.authorization[iface] = exsting;
      }
      exsting[operation] = rule;
    }
  }

  implement(handlers: Record<Handler, Step[]>): Application {
    for (const handler of Object.keys(handlers)) {
      const steps = handlers[handler as Handler];
      const [iface, oper] = handler.split("::");
      let pipelines = this.config.interfaces[iface];
      if (!pipelines) {
        pipelines = {};
        this.config.interfaces[iface] = pipelines;
      }
      pipelines[oper] = {
        steps: steps,
      };
    }
    return this;
  }

  interface<T extends Operations>(name: string, arg: T): Handlers<T> {
    const ret: { [name: string]: Handler } = {};
    const pipelines: Pipelines = {};
    for (const key of Object.keys(arg)) {
      const steps = arg[key];
      ret[key] = (name + "::" + key) as Handler;
      if (steps != undefined && steps.length > 0) {
        pipelines[key] = {
          steps: arg[key],
        };
      }
    }
    this.config.interfaces[name] = pipelines;
    return ret as Handlers<T>;
  }

  provider<T extends Operations>(name: string, arg: T): Handlers<T> {
    const ret: { [name: string]: Handler } = {};
    const pipelines: Pipelines = {};
    for (const key of Object.keys(arg)) {
      ret[key] = (name + "::" + key) as Handler;
      pipelines[key] = {
        steps: arg[key],
      };
    }
    this.config.providers[name] = pipelines;
    return ret as Handlers<T>;
  }

  error(name: string, template: ErrorTemplate): Application {
    this.config.errors[name] = template;
    return this;
  }

  filters(...comps: Component<unknown>[]): Application {
    this.config.filters.push(...comps);
    return this;
  }

  errors(map: { [name: string]: ErrorTemplate }): Application {
    for (const name of Object.keys(map)) {
      this.config.errors[name] = map[name];
    }
    return this;
  }

  asYAML(): string {
    const r = this.config as unknown as Record<string, unknown>;
    removeEmpty(this.config.resiliency as unknown as Record<string, unknown>);
    removeUndefined(r);
    removeEmpty(r);
    return YAML.stringify(r, { noRefs: true }).trim();
  }

  emit(): void {
    console.log(this.asYAML());
  }
}

function removeEmpty(rec: Record<string, unknown>) {
  for (const key of Object.keys(rec)) {
    const val = rec[key];
    if (
      val instanceof Object &&
      Object.keys(val as Record<string, unknown>).length == 0
    ) {
      delete rec[key];
    } else if (val instanceof Array && (val as Array<unknown>).length == 0) {
      delete rec[key];
    }
  }
}

function removeUndefined(rec: Record<string, unknown>) {
  for (const key of Object.keys(rec)) {
    const val = rec[key];
    if (val == undefined) {
      delete rec[key];
    }
    if (val instanceof Object) {
      removeUndefined(val as Record<string, unknown>);
    }
  }
}

//////////////////

interface Resiliency {
  timeouts: { [name: string]: Duration };
  retries: { [name: string]: Backoff };
  circuitBreakers: { [name: string]: CircuitBreaker };
}

type Backoff = ConstantBackoffWrapper | ExponentialBackoffWrapper;

export function constantBackoff(
  dur: string,
  maxRetries?: number,
): ConstantBackoffWrapper {
  return {
    constant: {
      duration: duration(dur),
      maxRetries,
    },
  };
}

interface ConstantBackoffWrapper {
  constant: ConstantBackoff;
}

export function exponentialBackoff(
  config: ExponentialBackoff,
): ExponentialBackoffWrapper {
  return {
    exponential: config,
  };
}

interface ExponentialBackoffWrapper {
  exponential: ExponentialBackoff;
}

export interface RetryConfig {
  maxRetries?: number;
}

export interface ConstantBackoff extends RetryConfig {
  duration: Duration;
}

export interface ExponentialBackoff extends RetryConfig {
  initialInterval?: Duration;
  randomizationFactor?: number;
  multiplier?: number;
  maxInterval?: Duration;
  maxElapsedTime?: Duration;
}

export interface CircuitBreaker extends RetryConfig {
  maxRequests?: number;
  interval?: Duration;
  timeout?: Duration;
  trip?: ValueExpr;
}

export type TimeoutRef = string & { __desc: "Timeout" };
export type RetryRef = string & { __desc: "Retry" };
export type CircuitBreakerRef = string & { __desc: "Circuit breaker" };

export interface ResiliencyGroup {
  timeout?: TimeoutRef;
  retry?: RetryRef;
  circuitBreaker?: CircuitBreakerRef;
}

export const unauthenticated: Unauthenticated = { unauthenticated: true };

export type Authorization = Unauthenticated | Secured;
export type Authorizations = { [key: string]: Authorization };

export interface Unauthenticated {
  unauthenticated: boolean;
}

export interface Secured {
  has?: string[];
  checks?: { [variable: string]: unknown };
  rules?: [Component<unknown>];
}

export function step(
  name: string,
  // deno-lint-ignore no-explicit-any
  comp: Component<any>,
  options: Partial<Step> = {},
): Step {
  return {
    name,
    ...comp,
    ...options,
  };
}

// deno-lint-ignore no-explicit-any
export type Step = StepT<any>;

export type StepT<T> = StepWith<T> | StepWithout;

export interface StepWithout extends ResiliencyGroup {
  name: string;
  uses: string;
  returns?: string;
}

export interface StepWith<T> extends Component<T>, ResiliencyGroup {
  name: string;
  returns?: string;
}

export interface Component<T> {
  uses: string;
  with: T | undefined;
}

export function component(uses: string, config: unknown): Component<unknown> {
  return {
    uses: uses,
    with: config,
  };
}
