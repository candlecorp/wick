// deno-lint-ignore-file no-explicit-any
/*
 * Copyright 2022 The NanoBus Authors.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

import { Application, Component, Module } from "../nanobus.ts";
import { RestV1 } from "./transport_rest.ts";
import { HttpServerV1 } from "./transport_server.ts";
import { OAuth2V1, OAuth2V1Config } from "./transport_oauth2.ts";
import { StaticPath, StaticV1, StaticV1Config } from "./transport_static.ts";
import { CorsV0, CorsV0Config } from "./transport_cors.ts";
import { AddRoute, RouterV1 } from "./transport_router.ts";

export const standardErrors = {
  not_found: {
    type: "NotFound",
    code: "not_found",
    title: "Resource not found",
    message: "Resource with id {{ .key }} was not found",
  },
  permission_denied: {
    type: "PermissionDenied",
    code: "permission_denied",
    title: "Permission denied",
    message:
      "You don't have permission to access this resource or to perform the operation.",
  },
  unauthenticated: {
    type: "Unauthenticated",
    code: "unauthenticated",
    title: "Unauthenticated",
    message: "You must be logged in to perform the operation.",
  },
};

export interface HttpModuleOptions {
  oauth2?: OAuth2V1Config;
  static?: StaticV1Config;
  cors?: CorsV0Config;
}

export class RestModule implements Module {
  private address: string;
  private options: HttpModuleOptions;

  constructor(address: string, options: HttpModuleOptions = {}) {
    this.address = address;
    this.options = options;
  }

  initialize(app: Application): void {
    const routers: Component<any>[] = [];
    const middleware: Component<any>[] = [];

    if (this.options.oauth2) {
      routers.push(OAuth2V1(this.options.oauth2));
    }

    routers.push(RestV1({
      documentation: {
        swaggerUI: true,
        postman: true,
        restClient: true,
      },
    }));

    if (this.options.static) {
      routers.push(StaticV1(this.options.static));
    }

    if (this.options.cors) {
      middleware.push(
        CorsV0({
          // Typical settings for REST API servers.
          allowedMethods: ["HEAD", "GET", "POST", "PUT", "PATCH", "DELETE"],
          allowCredentials: true,
          ...this.options.cors,
        }),
      );
    }

    app.transport(
      "http",
      HttpServerV1({
        address: this.address,
        middleware: middleware,
        routers: routers,
      }),
    );

    app.errors(standardErrors);
  }
}

export class HttpModule implements Module {
  private address: string;
  private options: HttpModuleOptions;
  private routes: AddRoute[];

  constructor(address: string, options: HttpModuleOptions, ...routes: AddRoute[]) {
    this.address = address;
    this.options = options;
    this.routes = routes;
  }

  initialize(app: Application): void {
    const routers: Component<any>[] = [];
    const middleware: Component<any>[] = [];

    if (this.options.static) {
      routers.push(StaticV1(this.options.static));
    }

    routers.push(RouterV1({
      routes: this.routes,
    }));

    if (this.options.cors) {
      middleware.push(
        CorsV0({
          // Typical settings for REST API servers.
          allowedMethods: ["HEAD", "GET", "POST", "PUT", "PATCH", "DELETE"],
          allowCredentials: true,
          ...this.options.cors,
        }),
      );
    }

    app.transport(
      "http",
      HttpServerV1({
        address: ":8080",
        routers: routers,
      }),
    );

    app.errors(standardErrors);
  }
}

export function singlePageAppPaths(
  dir: string,
  ...assetPaths: string[]
): StaticPath[] {
  const paths: StaticPath[] = [];
  assetPaths.forEach((path) => {
    if (!path.startsWith("/")) {
      path = "/" + path;
    }
    paths.push({
      dir: `${dir}${path}`,
      path: path,
      strip: path,
    });
  });
  paths.push({
    file: `${dir}/index.html`,
    path: "/",
  });
  return paths;
}
