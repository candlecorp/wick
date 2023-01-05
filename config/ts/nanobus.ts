// deno-lint-ignore-file no-explicit-any
/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

import * as YAML from "https://deno.land/std@0.167.0/encoding/yaml.ts";
import { Duration as Dur } from "https://deno.land/x/durationjs@v4.0.0/mod.ts";
import { Claims } from "./claims.ts";

export type ResourceRef = string & { __desc: "Resource" };
export type Duration = string & { __desc: "Duration" };
export type ValueExpr = any;
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
  readonly imports: { [key: string]: Ref };
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

export interface FlowClass {
  done(): Step[];
}

class FlowBuilder<T> {
  private steps: Step[] = [];

  constructor(steps: Step[] = []) {
    this.steps = steps;
  }

  then<O>(
    name: string,
    fn: ($: T) => Partial<ComponentWithOutput<unknown, O>>,
    ...options: Partial<StepOptionsT<O>>[]
  ): FlowBuilder<O> {
    const c = fn(propertyProxy.$ as T);
    const component = c as Component<unknown>;
    const s: StepT<unknown> = {
      name,
      ...component,
    };
    for (const opt of options) {
      if (opt.timeout) {
        s.timeout = opt.timeout;
      }
      if (opt.retry) {
        s.retry = opt.retry;
      }
      if (opt.circuitBreaker) {
        s.circuitBreaker = opt.circuitBreaker;
      }
      if (opt.returns) {
        s.returns = opt.returns as string;
      }
    }
    return new FlowBuilder<O>([...this.steps, s]);
  }

  done(): Step[] {
    return this.steps;
  }
}

export function given<T>(_input: T): FlowBuilder<T> {
  return new FlowBuilder<T>();
}

export type Flow<T> = (data: Context<T>, vars: any) => Step[] | FlowClass;

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
      imports: {},
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

  import<T>(
    instanceId: string,
    iota: Iota<T>,
    options: IncludeOptions = {},
  ): T {
    this.config.imports[instanceId] = {
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

  register(
    handlers: Record<string, Handler>,
    iface: Record<string, Flow<any>>,
  ): Application {
    const impls: Record<Handler, Step[]> = {};
    for (const funcName of Object.keys(iface)) {
      impls[handlers[funcName]] = getSteps(iface[funcName]);
    }
    this.implement(impls);
    return this;
  }

  provide(
    handlers: Record<string, Handler>,
    iface: Record<string, Flow<any>>,
  ): Application {
    const impls: Record<Handler, Step[]> = {};
    for (const funcName of Object.keys(iface)) {
      impls[handlers[funcName]] = getSteps(iface[funcName]);
    }
    this.internal(impls);
    return this;
  }

  authorize(
    handlers: Record<string, Handler>,
    iface: Record<string, Authorization>,
  ): Application {
    const auths: Record<Handler, Authorization> = {};
    for (const funcName of Object.keys(iface)) {
      auths[handlers[funcName]] = iface[funcName];
    }
    this.authorizations(auths);
    return this;
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

  internal(handlers: Record<Handler, Step[]>): Application {
    for (const handler of Object.keys(handlers)) {
      const steps = handlers[handler as Handler];
      const [iface, oper] = handler.split("::");
      let pipelines = this.config.providers[iface];
      if (!pipelines) {
        pipelines = {};
        this.config.providers[iface] = pipelines;
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

export interface StepOptionsT<T> extends ResiliencyGroup {
  returns?: T;
}

export interface StepOptions extends ResiliencyGroup {
  returns?: any;
}

export function returns<T>(value: T): Partial<StepOptionsT<T>> {
  return {
    returns: value,
  };
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

export type Step = StepT<any>;

export interface StepT<T> extends Component<T>, ResiliencyGroup {
  name: string;
  returns?: string;
}

export interface Component<T> {
  uses: string;
  with?: T;
}

export interface ComponentWithOutput<T, O> extends Component<T> {
  returns?: O;
}

export type Response<T> = ComponentWithOutput<unknown, T>

export function component(uses: string, config: unknown): Component<unknown> {
  return {
    uses: uses,
    with: config,
  };
}

//////////////

export interface Data<T> {
  /** Security claims */
  claims: Claims;
  /** Input from the request */
  input: T;
  /** Custom variables */
  [variable: string]: any;
}

export interface Context<T> extends Data<T> {
  flow: FlowBuilder<T>;
}

const handler = {
  get(target: any, prop: any, _receiver: unknown): unknown {
    const value = target[prop];
    if (value instanceof Function) {
      return (...args: unknown[]) => {
        //return value.apply(this === receiver ? target : this, args);
        return value.apply(target, args);
      };
    }
    if (value) {
      return value;
    }

    let parent: string[] = [];
    if (target instanceof DynamicProperty) {
      parent = (target as DynamicProperty).prop;
    }

    if (parent.length > 0 && !isNaN(prop)) {
      const addIndexer = [...parent];
      addIndexer[addIndexer.length - 1] += "[" + prop + "]";
      return new Proxy(new DynamicProperty(addIndexer), handler);
    }

    return new Proxy(new DynamicProperty([...parent, prop]), handler);
  },
};

export class DynamicProperty {
  prop: string[];

  constructor(prop: string[]) {
    this.prop = prop;
  }

  toString(): string {
    return this.prop.join(".");
  }

  get [Symbol.toStringTag]() {
    return this.toString();
  }

  [Symbol.toPrimitive](hint: any) {
    if (hint === "string") {
      return this.toString();
    }
    return null;
  }
}

export const propertyProxy = new Proxy({}, handler);

export class StepBuilder<T> implements StepT<T> {
  name: string;
  uses: string;
  with?: T;
  timeout?: TimeoutRef;
  retry?: RetryRef;
  circuitBreaker?: CircuitBreakerRef;
  returns?: string;

  constructor(name: string, comp: Component<T>) {
    this.name = name;
    this.uses = comp.uses;
    this.with = comp.with;
  }

  withTimeout(timeout: TimeoutRef | undefined): StepBuilder<T> {
    this.timeout = timeout;
    return this;
  }

  withRetry(retry: RetryRef | undefined): StepBuilder<T> {
    this.retry = retry;
    return this;
  }

  withResiliency(resiliency: ResiliencyGroup): StepBuilder<T> {
    if (resiliency.timeout) {
      this.timeout = resiliency.timeout;
    }
    if (resiliency.retry) {
      this.retry = resiliency.retry;
    }
    if (resiliency.circuitBreaker) {
      this.circuitBreaker = resiliency.circuitBreaker;
    }
    return this;
  }

  withCircuitBreaker(
    circuitBreaker: CircuitBreakerRef | undefined,
  ): StepBuilder<T> {
    this.circuitBreaker = circuitBreaker;
    return this;
  }

  return(variable: any): StepBuilder<T> {
    const v = variable as unknown;
    if (v instanceof DynamicProperty) {
      variable = (v as DynamicProperty).toString();
    }
    this.returns = "" + variable;
    return this;
  }
}

export function step<T>(
  name: string,
  comp: Component<T>,
  options?: StepOptions,
): StepBuilder<T> {
  replaceFakes(comp as unknown as Record<string, unknown>);
  const builder = new StepBuilder(name, comp);
  if (options) {
    builder.withResiliency(options);
    if (options.returns) {
      builder.return(options?.returns);
    }
  }

  return builder;
}

export function getSteps(f: Flow<unknown>): Step[] {
  const context = new Proxy({
    flow: new FlowBuilder<unknown>(),
  }, handler) as Context<unknown>;
  const stepsOrFlow = f(context, propertyProxy);
  let steps: Step[];
  if (stepsOrFlow instanceof FlowBuilder) {
    steps = (stepsOrFlow as FlowClass).done();
  } else {
    steps = stepsOrFlow as Step[];
  }
  for (let i = 0; i < steps.length; i++) {
    steps[i] = { ...steps[i] };
    scrub(steps[i]);
  }
  return steps;
}

export function toExpr(input: unknown): string {
  if (input instanceof DynamicProperty) {
    input = (input as DynamicProperty).toString();
  }
  return input as string || "nil";
}

export function toDataExpr(input: unknown): string {
  let str = toExpr(input);
  if (str.startsWith("$.")) {
    str = "pipe" + str.substring(1);
  }
  return str;
}

export function scrub(data: unknown): void {
  replaceFakes(data as Record<string, unknown>);
  removeUndefined(data as Record<string, unknown>);
  trimStrings(data as Record<string, unknown>);
}

function replaceFakes(rec: Record<string, unknown>) {
  for (const key of Object.keys(rec)) {
    const val = rec[key];
    if (val instanceof DynamicProperty) {
      rec[key] = (val as DynamicProperty).toString();
    }
    if (val instanceof Object) {
      replaceFakes(val as Record<string, unknown>);
    }
    if (val instanceof String) {
      rec[key] = (val as string).trim();
    }
    if (val instanceof Array) {
      const ary = val as Array<any>;
      for (let i = 0; i < ary.length; i++) {
        const item = ary[i];
        if (item instanceof DynamicProperty) {
          ary[i] = (item as DynamicProperty).toString();
        }
        if (item instanceof Object) {
          replaceFakes(item as Record<string, unknown>);
        }
      }
    }
  }
}

function trimStrings(rec: Record<string, unknown>) {
  for (const key of Object.keys(rec)) {
    const val = rec[key];
    if (typeof val === "string" || val instanceof String) {
      rec[key] = (val as string).trim();
    } else if (val instanceof Array) {
      const ary = val as Array<any>;
      for (let i = 0; i < ary.length; i++) {
        const item = ary[i];
        if (typeof item === "string" || item instanceof String) {
          ary[i] = (item as string).trim();
        }
        if (item instanceof Object) {
          trimStrings(item as Record<string, unknown>);
        }
      }
    } else if (val instanceof Object) {
      trimStrings(val as Record<string, unknown>);
    }
  }
}
