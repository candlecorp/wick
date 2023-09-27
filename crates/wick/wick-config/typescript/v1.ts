

export interface HasKind {
  getKind: () => string;
}


    
    
    
    

    
    
    
export type WickConfig =
      AppConfiguration|ComponentConfiguration|TypesConfiguration|TestConfiguration|LockdownConfiguration
    ;
    

    
    
    
    export type LocationReference =

  string;


    
    
    
    export type BoundIdentifier =

  string;


    
    
    
    export type LiquidJsonValue =
any;
  


    
    
    
    export type LiquidTemplate =

  string;


    
    
    
    export type Glob =

  string;




export class AppConfiguration implements HasKind {
 // The application&#x27;s name. 
      _name : string ;
 // Associated metadata for this application. 
      _metadata : Metadata| undefined =  undefined;
 // Details about the package for this application. 
      _package : PackageDefinition| undefined =  undefined;
 // Resources and configuration that the application and its components can access. 
      _resources : ResourceBinding[] =  [];
 // Components that to import and make available to the application. 
      _import : ImportBinding[] =  [];
 // Triggers to load and instantiate to drive the application&#x27;s behavior. 
      _triggers : TriggerDefinition[] =  [];
    constructor (
name:
 string,
      ) {
          this._name = name;
    }

name(value: string) : AppConfiguration {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
metadata(value: Metadata| undefined) : AppConfiguration {
      this._metadata = value;
      return this;
    }
    getMetadata() : Metadata| undefined {
      return this._metadata;

    }
package(value: PackageDefinition| undefined) : AppConfiguration {
      this._package = value;
      return this;
    }
    getPackage() : PackageDefinition| undefined {
      return this._package;

    }
resources(value: ResourceBinding[]) : AppConfiguration {
      this._resources = value;
      return this;
    }
    getResources() : ResourceBinding[] {
      return this._resources;

    }
import(value: ImportBinding[]) : AppConfiguration {
      this._import = value;
      return this;
    }
    getImport() : ImportBinding[] {
      return this._import;

    }
triggers(value: TriggerDefinition[]) : AppConfiguration {
      this._triggers = value;
      return this;
    }
    getTriggers() : TriggerDefinition[] {
      return this._triggers;

    }

    getKind() : string {
      return "wick/app@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/app@v1",
name: this._name,metadata: this._metadata,package: this._package,resources: this._resources,import: this._import,triggers: this._triggers,      }

    }
}

    
    
    
    



export class Metadata implements HasKind {
 // The version of the artifact. 
      _version : string ="";
 // A list of the authors. 
      _authors : string[] =  [];
 // A list of any vendors associated with the artifact. 
      _vendors : string[] =  [];
 // A short description. 
      _description : string| undefined =  undefined;
 // Where to find documentation. 
      _documentation : string| undefined =  undefined;
 // The license(s) for the artifact. 
      _licenses : string[] =  [];
 // An icon to associate with the artifact. 
      _icon : string| undefined =  undefined;
    constructor (
      ) {
    }

version(value: string) : Metadata {
      this._version = value;
      return this;
    }
    getVersion() : string {
      return this._version;

    }
authors(value: string[]) : Metadata {
      this._authors = value;
      return this;
    }
    getAuthors() : string[] {
      return this._authors;

    }
vendors(value: string[]) : Metadata {
      this._vendors = value;
      return this;
    }
    getVendors() : string[] {
      return this._vendors;

    }
description(value: string| undefined) : Metadata {
      this._description = value;
      return this;
    }
    getDescription() : string| undefined {
      return this._description;

    }
documentation(value: string| undefined) : Metadata {
      this._documentation = value;
      return this;
    }
    getDocumentation() : string| undefined {
      return this._documentation;

    }
licenses(value: string[]) : Metadata {
      this._licenses = value;
      return this;
    }
    getLicenses() : string[] {
      return this._licenses;

    }
icon(value: string| undefined) : Metadata {
      this._icon = value;
      return this;
    }
    getIcon() : string| undefined {
      return this._icon;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
version: this._version,authors: this._authors,vendors: this._vendors,description: this._description,documentation: this._documentation,licenses: this._licenses,icon: this._icon,      }

    }
}

    
    
    
    



export class PackageDefinition implements HasKind {
 // The list of files and folders to be included with the package. 
      _files : string[] =  [];
 // Configuration for publishing the package to a registry. 
      _registry : RegistryDefinition| undefined =  undefined;
    constructor (
      ) {
    }

files(value: string[]) : PackageDefinition {
      this._files = value;
      return this;
    }
    getFiles() : string[] {
      return this._files;

    }
registry(value: RegistryDefinition| undefined) : PackageDefinition {
      this._registry = value;
      return this;
    }
    getRegistry() : RegistryDefinition| undefined {
      return this._registry;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
files: this._files,registry: this._registry,      }

    }
}

    
    
    
    



export class RegistryDefinition implements HasKind {
 // The registry to publish to, e.g. registry.candle.dev 
      _host : string ="";
 // The namespace on the registry. e.g.: [*your username*] 
      _namespace : string ="";
    constructor (
      ) {
    }

host(value: string) : RegistryDefinition {
      this._host = value;
      return this;
    }
    getHost() : string {
      return this._host;

    }
namespace(value: string) : RegistryDefinition {
      this._namespace = value;
      return this;
    }
    getNamespace() : string {
      return this._namespace;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
host: this._host,namespace: this._namespace,      }

    }
}

    
    
    
    



export class ResourceBinding implements HasKind {
 // The name of the binding. 
      _name : string ;
 // The resource to bind to. 
      _resource : ResourceDefinition ;
    constructor (
name:
 string,
resource:
 ResourceDefinition,
      ) {
          this._name = name;
          this._resource = resource;
    }

name(value: string) : ResourceBinding {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
resource(value: ResourceDefinition) : ResourceBinding {
      this._resource = value;
      return this;
    }
    getResource() : ResourceDefinition {
      return this._resource;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,resource: this._resource,      }

    }
}

    
    
    
    



export class ImportBinding implements HasKind {
 // The name of the binding. 
      _name : string ;
 // The import to bind to. 
      _component : ImportDefinition ;
    constructor (
name:
 string,
component:
 ImportDefinition,
      ) {
          this._name = name;
          this._component = component;
    }

name(value: string) : ImportBinding {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
component(value: ImportDefinition) : ImportBinding {
      this._component = value;
      return this;
    }
    getComponent() : ImportDefinition {
      return this._component;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,component: this._component,      }

    }
}

    
    
    
    

    
    
    
export type ResourceDefinition =
      TcpPort|UdpPort|Url|Volume
    ;
    



export class TcpPort implements HasKind {
 // The port to bind to. 
      _port : LiquidTemplate ;
 // The address to bind to. 
      _address : LiquidTemplate ;
    constructor (
port:
 LiquidTemplate,
address:
 LiquidTemplate,
      ) {
          this._port = port;
          this._address = address;
    }

port(value: LiquidTemplate) : TcpPort {
      this._port = value;
      return this;
    }
    getPort() : LiquidTemplate {
      return this._port;

    }
address(value: LiquidTemplate) : TcpPort {
      this._address = value;
      return this;
    }
    getAddress() : LiquidTemplate {
      return this._address;

    }

    getKind() : string {
      return "wick/resource/tcpport@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/resource/tcpport@v1",
port: this._port,address: this._address,      }

    }
}

    
    
    
    



export class UdpPort implements HasKind {
 // The port to bind to. 
      _port : LiquidTemplate ;
 // The address to bind to. 
      _address : LiquidTemplate ;
    constructor (
port:
 LiquidTemplate,
address:
 LiquidTemplate,
      ) {
          this._port = port;
          this._address = address;
    }

port(value: LiquidTemplate) : UdpPort {
      this._port = value;
      return this;
    }
    getPort() : LiquidTemplate {
      return this._port;

    }
address(value: LiquidTemplate) : UdpPort {
      this._address = value;
      return this;
    }
    getAddress() : LiquidTemplate {
      return this._address;

    }

    getKind() : string {
      return "wick/resource/udpport@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/resource/udpport@v1",
port: this._port,address: this._address,      }

    }
}

    
    
    
    



export class Volume implements HasKind {
 // The path. 
      _path : LiquidTemplate ;
    constructor (
path:
 LiquidTemplate,
      ) {
          this._path = path;
    }

path(value: LiquidTemplate) : Volume {
      this._path = value;
      return this;
    }
    getPath() : LiquidTemplate {
      return this._path;

    }

    getKind() : string {
      return "wick/resource/volume@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/resource/volume@v1",
path: this._path,      }

    }
}

    
    
    
    



export class Url implements HasKind {
 // The url string. 
      _url : LiquidTemplate ;
    constructor (
url:
 LiquidTemplate,
      ) {
          this._url = url;
    }

url(value: LiquidTemplate) : Url {
      this._url = value;
      return this;
    }
    getUrl() : LiquidTemplate {
      return this._url;

    }

    getKind() : string {
      return "wick/resource/url@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/resource/url@v1",
url: this._url,      }

    }
}

    
    
    
    

    
    
    
export type TriggerDefinition =
      CliTrigger|HttpTrigger|TimeTrigger|WasmCommandTrigger
    ;
    



export class WasmCommandTrigger implements HasKind {
 // The component to execute 
      _reference : string ;
 // Volumes to expose to the component. 
      _volumes : ExposedVolume[] =  [];
    constructor (
reference:
 string,
      ) {
          this._reference = reference;
    }

reference(value: string) : WasmCommandTrigger {
      this._reference = value;
      return this;
    }
    getReference() : string {
      return this._reference;

    }
volumes(value: ExposedVolume[]) : WasmCommandTrigger {
      this._volumes = value;
      return this;
    }
    getVolumes() : ExposedVolume[] {
      return this._volumes;

    }

    getKind() : string {
      return "wick/trigger/wasm-command@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/trigger/wasm-command@v1",
reference: this._reference,volumes: this._volumes,      }

    }
}

    
    
    
    



export class CliTrigger implements HasKind {
 // The operation that will act as the main entrypoint for this trigger. 
      _operation :string | ComponentOperationExpression ;
    constructor (
operation:
string | ComponentOperationExpression,
      ) {
          this._operation = operation;
    }

operation(value: ComponentOperationExpression) : CliTrigger {
      this._operation = value;
      return this;
    }
    getOperation() :string | ComponentOperationExpression {
      return this._operation;

    }

    getKind() : string {
      return "wick/trigger/cli@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/trigger/cli@v1",
operation: this._operation,      }

    }
}

    
    
    
    



export class TimeTrigger implements HasKind {
 // The schedule to run the trigger with. 
      _schedule : Schedule ;
 // The operation to execute on the schedule. 
      _operation :string | ComponentOperationExpression ;
 // Values passed to the operation as inputs 
      _payload : OperationInput[] ;
    constructor (
schedule:
 Schedule,
operation:
string | ComponentOperationExpression,
payload:
 OperationInput[],
      ) {
          this._schedule = schedule;
          this._operation = operation;
          this._payload = payload;
    }

schedule(value: Schedule) : TimeTrigger {
      this._schedule = value;
      return this;
    }
    getSchedule() : Schedule {
      return this._schedule;

    }
operation(value: ComponentOperationExpression) : TimeTrigger {
      this._operation = value;
      return this;
    }
    getOperation() :string | ComponentOperationExpression {
      return this._operation;

    }
payload(value: OperationInput[]) : TimeTrigger {
      this._payload = value;
      return this;
    }
    getPayload() : OperationInput[] {
      return this._payload;

    }

    getKind() : string {
      return "wick/trigger/time@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/trigger/time@v1",
schedule: this._schedule,operation: this._operation,payload: this._payload,      }

    }
}

    
    
    
    



export class OperationInput implements HasKind {
 // The name of the input. 
      _name : string ;
 // The value to pass. 
      _value : any ;
    constructor (
name:
 string,
value:
 any,
      ) {
          this._name = name;
          this._value = value;
    }

name(value: string) : OperationInput {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
value(value: any) : OperationInput {
      this._value = value;
      return this;
    }
    getValue() : any {
      return this._value;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,value: this._value,      }

    }
}

    
    
    
    



export class Schedule implements HasKind {
 // Schedule in cron format with second precision. See [cron.help](https://cron.help) for more information. 
      _cron : string ;
 // repeat &#x60;n&#x60; times. Use &#x60;0&#x60; to repeat indefinitely 
      _repeat : number =0;
    constructor (
cron:
 string,
      ) {
          this._cron = cron;
    }

cron(value: string) : Schedule {
      this._cron = value;
      return this;
    }
    getCron() : string {
      return this._cron;

    }
repeat(value: number) : Schedule {
      this._repeat = value;
      return this;
    }
    getRepeat() : number {
      return this._repeat;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
cron: this._cron,repeat: this._repeat,      }

    }
}

    
    
    
    



export class ComponentOperationExpression implements HasKind {
 // The component that exports the operation. 
      _component :string | ComponentDefinition ;
 // The operation name. 
      _name : string ;
 // Configuration to pass to this operation on invocation. 
      _with :   Record<string,LiquidJsonValue>| undefined =  undefined;
 // Timeout (in milliseconds) to wait for the operation to complete. Use 0 to wait indefinitely. 
      _timeout : number| undefined =  undefined;
    constructor (
component:
string | ComponentDefinition,
name:
 string,
      ) {
          this._component = component;
          this._name = name;
    }

component(value: ComponentDefinition) : ComponentOperationExpression {
      this._component = value;
      return this;
    }
    getComponent() :string | ComponentDefinition {
      return this._component;

    }
name(value: string) : ComponentOperationExpression {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
with(value:   Record<string,LiquidJsonValue>| undefined) : ComponentOperationExpression {
      this._with = value;
      return this;
    }
    getWith() :   Record<string,LiquidJsonValue>| undefined {
      return this._with;

    }
timeout(value: number| undefined) : ComponentOperationExpression {
      this._timeout = value;
      return this;
    }
    getTimeout() : number| undefined {
      return this._timeout;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
component: this._component,name: this._name,with: this._with,timeout: this._timeout,      }

    }
}

    
    
    
    



export class HttpTrigger implements HasKind {
 // The TcpPort resource to listen on for connections. 
      _resource : BoundIdentifier ;
 // The router to handle incoming requests 
      _routers : HttpRouter[] =  [];
    constructor (
resource:
 BoundIdentifier,
      ) {
          this._resource = resource;
    }

resource(value: BoundIdentifier) : HttpTrigger {
      this._resource = value;
      return this;
    }
    getResource() : BoundIdentifier {
      return this._resource;

    }
routers(value: HttpRouter[]) : HttpTrigger {
      this._routers = value;
      return this;
    }
    getRouters() : HttpRouter[] {
      return this._routers;

    }

    getKind() : string {
      return "wick/trigger/http@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/trigger/http@v1",
resource: this._resource,routers: this._routers,      }

    }
}

    
    
    
    

    
    
    
export type HttpRouter =
      RawRouter|RestRouter|StaticRouter|ProxyRouter
    ;
    



export class ProxyRouter implements HasKind {
 // The path that this router will trigger for. 
      _path : string ;
 // Middleware operations for this router. 
      _middleware : Middleware| undefined =  undefined;
 // The URL resource to proxy to. 
      _url : BoundIdentifier ;
 // Whether or not to strip the router&#x27;s path from the proxied request. 
      _stripPath : boolean =false;
    constructor (
path:
 string,
url:
 BoundIdentifier,
      ) {
          this._path = path;
          this._url = url;
    }

path(value: string) : ProxyRouter {
      this._path = value;
      return this;
    }
    getPath() : string {
      return this._path;

    }
middleware(value: Middleware| undefined) : ProxyRouter {
      this._middleware = value;
      return this;
    }
    getMiddleware() : Middleware| undefined {
      return this._middleware;

    }
url(value: BoundIdentifier) : ProxyRouter {
      this._url = value;
      return this;
    }
    getUrl() : BoundIdentifier {
      return this._url;

    }
stripPath(value: boolean) : ProxyRouter {
      this._stripPath = value;
      return this;
    }
    getStripPath() : boolean {
      return this._stripPath;

    }

    getKind() : string {
      return "wick/router/proxy@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/router/proxy@v1",
path: this._path,middleware: this._middleware,url: this._url,strip_path: this._stripPath,      }

    }
}

    
    
    
    



export class RestRouter implements HasKind {
 // The path that this router will trigger for. 
      _path : string ;
 // Additional tools and services to enable. 
      _tools : Tools| undefined =  undefined;
 // Middleware operations for this router. 
      _middleware : Middleware| undefined =  undefined;
 // The routes to serve and operations that handle them. 
      _routes : Route[] =  [];
 // Information about the router to use when generating documentation and other tools. 
      _info : Info| undefined =  undefined;
    constructor (
path:
 string,
      ) {
          this._path = path;
    }

path(value: string) : RestRouter {
      this._path = value;
      return this;
    }
    getPath() : string {
      return this._path;

    }
tools(value: Tools| undefined) : RestRouter {
      this._tools = value;
      return this;
    }
    getTools() : Tools| undefined {
      return this._tools;

    }
middleware(value: Middleware| undefined) : RestRouter {
      this._middleware = value;
      return this;
    }
    getMiddleware() : Middleware| undefined {
      return this._middleware;

    }
routes(value: Route[]) : RestRouter {
      this._routes = value;
      return this;
    }
    getRoutes() : Route[] {
      return this._routes;

    }
info(value: Info| undefined) : RestRouter {
      this._info = value;
      return this;
    }
    getInfo() : Info| undefined {
      return this._info;

    }

    getKind() : string {
      return "wick/router/rest@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/router/rest@v1",
path: this._path,tools: this._tools,middleware: this._middleware,routes: this._routes,info: this._info,      }

    }
}

    
    
    
    



export class Route implements HasKind {
 // The path to serve this route from. See [URI documentation](/docs/configuration/uri) for more information on specifying query and path parameters. 
      _subPath : string ;
 // The operation that will act as the main entrypoint for this route. 
      _operation :string | ComponentOperationExpression ;
 // The HTTP methods to serve this route for. 
      _methods : HttpMethod[] =  [];
 // The unique ID of the route, used for documentation and tooling. 
      _id : string| undefined =  undefined;
 // A short description of the route. 
      _description : string| undefined =  undefined;
 // A longer description of the route. 
      _summary : string| undefined =  undefined;
    constructor (
sub_path:
 string,
operation:
string | ComponentOperationExpression,
      ) {
          this._subPath = sub_path;
          this._operation = operation;
    }

subPath(value: string) : Route {
      this._subPath = value;
      return this;
    }
    getSubPath() : string {
      return this._subPath;

    }
operation(value: ComponentOperationExpression) : Route {
      this._operation = value;
      return this;
    }
    getOperation() :string | ComponentOperationExpression {
      return this._operation;

    }
methods(value: HttpMethod[]) : Route {
      this._methods = value;
      return this;
    }
    getMethods() : HttpMethod[] {
      return this._methods;

    }
id(value: string| undefined) : Route {
      this._id = value;
      return this;
    }
    getId() : string| undefined {
      return this._id;

    }
description(value: string| undefined) : Route {
      this._description = value;
      return this;
    }
    getDescription() : string| undefined {
      return this._description;

    }
summary(value: string| undefined) : Route {
      this._summary = value;
      return this;
    }
    getSummary() : string| undefined {
      return this._summary;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
sub_path: this._subPath,operation: this._operation,methods: this._methods,id: this._id,description: this._description,summary: this._summary,      }

    }
}

    
    
    
    



export class Tools implements HasKind {
 // Set to true to generate an OpenAPI specification and serve it at *router_path*/openapi.json 
      _openapi : boolean =false;
    constructor (
      ) {
    }

openapi(value: boolean) : Tools {
      this._openapi = value;
      return this;
    }
    getOpenapi() : boolean {
      return this._openapi;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
openapi: this._openapi,      }

    }
}

    
    
    
    



export class Info implements HasKind {
 // The title of the API. 
      _title : string| undefined =  undefined;
 // A short description of the API. 
      _description : string| undefined =  undefined;
 // The terms of service for the API. 
      _tos : string| undefined =  undefined;
 // The contact information for the API. 
      _contact : Contact| undefined =  undefined;
 // The license information for the API. 
      _license : License| undefined =  undefined;
 // The version of the API. 
      _version : string ="";
 // The URL to the API&#x27;s terms of service. 
      _documentation : Documentation| undefined =  undefined;
    constructor (
      ) {
    }

title(value: string| undefined) : Info {
      this._title = value;
      return this;
    }
    getTitle() : string| undefined {
      return this._title;

    }
description(value: string| undefined) : Info {
      this._description = value;
      return this;
    }
    getDescription() : string| undefined {
      return this._description;

    }
tos(value: string| undefined) : Info {
      this._tos = value;
      return this;
    }
    getTos() : string| undefined {
      return this._tos;

    }
contact(value: Contact| undefined) : Info {
      this._contact = value;
      return this;
    }
    getContact() : Contact| undefined {
      return this._contact;

    }
license(value: License| undefined) : Info {
      this._license = value;
      return this;
    }
    getLicense() : License| undefined {
      return this._license;

    }
version(value: string) : Info {
      this._version = value;
      return this;
    }
    getVersion() : string {
      return this._version;

    }
documentation(value: Documentation| undefined) : Info {
      this._documentation = value;
      return this;
    }
    getDocumentation() : Documentation| undefined {
      return this._documentation;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
title: this._title,description: this._description,tos: this._tos,contact: this._contact,license: this._license,version: this._version,documentation: this._documentation,      }

    }
}

    
    
    
    



export class Documentation implements HasKind {
 // The URL to the API&#x27;s documentation. 
      _url : string| undefined =  undefined;
 // A short description of the documentation. 
      _description : string| undefined =  undefined;
    constructor (
      ) {
    }

url(value: string| undefined) : Documentation {
      this._url = value;
      return this;
    }
    getUrl() : string| undefined {
      return this._url;

    }
description(value: string| undefined) : Documentation {
      this._description = value;
      return this;
    }
    getDescription() : string| undefined {
      return this._description;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
url: this._url,description: this._description,      }

    }
}

    
    
    
    



export class License implements HasKind {
 // The name of the license. 
      _name : string ;
 // The URL to the license. 
      _url : string| undefined =  undefined;
    constructor (
name:
 string,
      ) {
          this._name = name;
    }

name(value: string) : License {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
url(value: string| undefined) : License {
      this._url = value;
      return this;
    }
    getUrl() : string| undefined {
      return this._url;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,url: this._url,      }

    }
}

    
    
    
    



export class Contact implements HasKind {
 // The name of the contact. 
      _name : string| undefined =  undefined;
 // The URL to the contact. 
      _url : string| undefined =  undefined;
 // The email address of the contact. 
      _email : string| undefined =  undefined;
    constructor (
      ) {
    }

name(value: string| undefined) : Contact {
      this._name = value;
      return this;
    }
    getName() : string| undefined {
      return this._name;

    }
url(value: string| undefined) : Contact {
      this._url = value;
      return this;
    }
    getUrl() : string| undefined {
      return this._url;

    }
email(value: string| undefined) : Contact {
      this._email = value;
      return this;
    }
    getEmail() : string| undefined {
      return this._email;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,url: this._url,email: this._email,      }

    }
}

    
    
    
    



export class StaticRouter implements HasKind {
 // The path that this router will trigger for. 
      _path : string ;
 // Middleware operations for this router. 
      _middleware : Middleware| undefined =  undefined;
 // The volume to serve static files from. 
      _volume : string ;
 // Fallback path (relative to volume &#x60;resource&#x60;) for files to serve in case of a 404. Useful for SPA&#x27;s. if volume resource is: /www and fallback: index.html, then a 404 will serve /www/index.html 
      _fallback : string| undefined =  undefined;
 // Whether or not to serve directory listings when a directory is requested. 
      _indexes : boolean =false;
    constructor (
path:
 string,
volume:
 string,
      ) {
          this._path = path;
          this._volume = volume;
    }

path(value: string) : StaticRouter {
      this._path = value;
      return this;
    }
    getPath() : string {
      return this._path;

    }
middleware(value: Middleware| undefined) : StaticRouter {
      this._middleware = value;
      return this;
    }
    getMiddleware() : Middleware| undefined {
      return this._middleware;

    }
volume(value: string) : StaticRouter {
      this._volume = value;
      return this;
    }
    getVolume() : string {
      return this._volume;

    }
fallback(value: string| undefined) : StaticRouter {
      this._fallback = value;
      return this;
    }
    getFallback() : string| undefined {
      return this._fallback;

    }
indexes(value: boolean) : StaticRouter {
      this._indexes = value;
      return this;
    }
    getIndexes() : boolean {
      return this._indexes;

    }

    getKind() : string {
      return "wick/router/static@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/router/static@v1",
path: this._path,middleware: this._middleware,volume: this._volume,fallback: this._fallback,indexes: this._indexes,      }

    }
}

    
    
    
    



export class RawRouter implements HasKind {
 // The path that this router will trigger for. 
      _path : string ;
 // Middleware operations for this router. 
      _middleware : Middleware| undefined =  undefined;
 // The codec to use when encoding/decoding data. 
      _codec : Codec| undefined =  undefined;
 // The operation that handles HTTP requests. 
      _operation :string | ComponentOperationExpression ;
    constructor (
path:
 string,
operation:
string | ComponentOperationExpression,
      ) {
          this._path = path;
          this._operation = operation;
    }

path(value: string) : RawRouter {
      this._path = value;
      return this;
    }
    getPath() : string {
      return this._path;

    }
middleware(value: Middleware| undefined) : RawRouter {
      this._middleware = value;
      return this;
    }
    getMiddleware() : Middleware| undefined {
      return this._middleware;

    }
codec(value: Codec| undefined) : RawRouter {
      this._codec = value;
      return this;
    }
    getCodec() : Codec| undefined {
      return this._codec;

    }
operation(value: ComponentOperationExpression) : RawRouter {
      this._operation = value;
      return this;
    }
    getOperation() :string | ComponentOperationExpression {
      return this._operation;

    }

    getKind() : string {
      return "wick/router/raw@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/router/raw@v1",
path: this._path,middleware: this._middleware,codec: this._codec,operation: this._operation,      }

    }
}

    
    
    
    



export class Middleware implements HasKind {
 // The middleware to apply to requests. 
      _request : ComponentOperationExpression[] =  [];
 // The middleware to apply to responses. 
      _response : ComponentOperationExpression[] =  [];
    constructor (
      ) {
    }

request(value: ComponentOperationExpression[]) : Middleware {
      this._request = value;
      return this;
    }
    getRequest() : ComponentOperationExpression[] {
      return this._request;

    }
response(value: ComponentOperationExpression[]) : Middleware {
      this._response = value;
      return this;
    }
    getResponse() : ComponentOperationExpression[] {
      return this._response;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
request: this._request,response: this._response,      }

    }
}

    
    
    
    



export class TypesConfiguration implements HasKind {
 // The name of this type. 
      _name : string| undefined =  undefined;
 // Associated metadata for this type. 
      _metadata : Metadata| undefined =  undefined;
 // Additional types to export and make available to the type. 
      _types : TypeDefinition[] =  [];
 // A list of operation signatures. 
      _operations : OperationDefinition[] =  [];
 // Details about the package for this types. 
      _package : PackageDefinition| undefined =  undefined;
    constructor (
      ) {
    }

name(value: string| undefined) : TypesConfiguration {
      this._name = value;
      return this;
    }
    getName() : string| undefined {
      return this._name;

    }
metadata(value: Metadata| undefined) : TypesConfiguration {
      this._metadata = value;
      return this;
    }
    getMetadata() : Metadata| undefined {
      return this._metadata;

    }
types(value: TypeDefinition[]) : TypesConfiguration {
      this._types = value;
      return this;
    }
    getTypes() : TypeDefinition[] {
      return this._types;

    }
operations(value: OperationDefinition[]) : TypesConfiguration {
      this._operations = value;
      return this;
    }
    getOperations() : OperationDefinition[] {
      return this._operations;

    }
package(value: PackageDefinition| undefined) : TypesConfiguration {
      this._package = value;
      return this;
    }
    getPackage() : PackageDefinition| undefined {
      return this._package;

    }

    getKind() : string {
      return "wick/types@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/types@v1",
name: this._name,metadata: this._metadata,types: this._types,operations: this._operations,package: this._package,      }

    }
}

    
    
    
    



export class TestConfiguration implements HasKind {
 // The name of this component. 
      _name : string| undefined =  undefined;
 // Configuration used to instantiate this component. 
      _with :   Record<string,LiquidJsonValue>| undefined =  undefined;
 // Unit tests to run against components and operations. 
      _cases : TestDefinition[] =  [];
    constructor (
      ) {
    }

name(value: string| undefined) : TestConfiguration {
      this._name = value;
      return this;
    }
    getName() : string| undefined {
      return this._name;

    }
with(value:   Record<string,LiquidJsonValue>| undefined) : TestConfiguration {
      this._with = value;
      return this;
    }
    getWith() :   Record<string,LiquidJsonValue>| undefined {
      return this._with;

    }
cases(value: TestDefinition[]) : TestConfiguration {
      this._cases = value;
      return this;
    }
    getCases() : TestDefinition[] {
      return this._cases;

    }

    getKind() : string {
      return "wick/tests@v1";
    }

    toJSON() : any {
      return {
name: this._name,with: this._with,cases: this._cases,      }

    }
}

    
    
    
    



export class LockdownConfiguration implements HasKind {
 // Associated metadata for this configuration. 
      _metadata : Metadata| undefined =  undefined;
 // Restrictions to apply to resources before an application or component can be run. 
      _resources : ResourceRestriction[] =  [];
    constructor (
      ) {
    }

metadata(value: Metadata| undefined) : LockdownConfiguration {
      this._metadata = value;
      return this;
    }
    getMetadata() : Metadata| undefined {
      return this._metadata;

    }
resources(value: ResourceRestriction[]) : LockdownConfiguration {
      this._resources = value;
      return this;
    }
    getResources() : ResourceRestriction[] {
      return this._resources;

    }

    getKind() : string {
      return "wick/lockdown@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/lockdown@v1",
metadata: this._metadata,resources: this._resources,      }

    }
}

    
    
    
    

    
    
    
export type ResourceRestriction =
      VolumeRestriction|UrlRestriction|TcpPortRestriction|UdpPortRestriction
    ;
    



export class VolumeRestriction implements HasKind {
 // The components this restriction applies to 
      _components : string[] =  [];
 // The volumes to allow 
      _allow : LiquidTemplate ;
    constructor (
allow:
 LiquidTemplate,
      ) {
          this._allow = allow;
    }

components(value: string[]) : VolumeRestriction {
      this._components = value;
      return this;
    }
    getComponents() : string[] {
      return this._components;

    }
allow(value: LiquidTemplate) : VolumeRestriction {
      this._allow = value;
      return this;
    }
    getAllow() : LiquidTemplate {
      return this._allow;

    }

    getKind() : string {
      return "wick/resource/volume@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/resource/volume@v1",
components: this._components,allow: this._allow,      }

    }
}

    
    
    
    



export class UrlRestriction implements HasKind {
 // The components this restriction applies to 
      _components : string[] =  [];
 // The URLs to allow 
      _allow : LiquidTemplate ;
    constructor (
allow:
 LiquidTemplate,
      ) {
          this._allow = allow;
    }

components(value: string[]) : UrlRestriction {
      this._components = value;
      return this;
    }
    getComponents() : string[] {
      return this._components;

    }
allow(value: LiquidTemplate) : UrlRestriction {
      this._allow = value;
      return this;
    }
    getAllow() : LiquidTemplate {
      return this._allow;

    }

    getKind() : string {
      return "wick/resource/url@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/resource/url@v1",
components: this._components,allow: this._allow,      }

    }
}

    
    
    
    



export class TcpPortRestriction implements HasKind {
 // The components this restriction applies to 
      _components : string[] =  [];
 // The address to allow 
      _address : LiquidTemplate ;
 // The port to allow 
      _port : LiquidTemplate ;
    constructor (
address:
 LiquidTemplate,
port:
 LiquidTemplate,
      ) {
          this._address = address;
          this._port = port;
    }

components(value: string[]) : TcpPortRestriction {
      this._components = value;
      return this;
    }
    getComponents() : string[] {
      return this._components;

    }
address(value: LiquidTemplate) : TcpPortRestriction {
      this._address = value;
      return this;
    }
    getAddress() : LiquidTemplate {
      return this._address;

    }
port(value: LiquidTemplate) : TcpPortRestriction {
      this._port = value;
      return this;
    }
    getPort() : LiquidTemplate {
      return this._port;

    }

    getKind() : string {
      return "wick/resource/tcpport@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/resource/tcpport@v1",
components: this._components,address: this._address,port: this._port,      }

    }
}

    
    
    
    



export class UdpPortRestriction implements HasKind {
 // The components this restriction applies to 
      _components : string[] =  [];
 // The address to allow 
      _address : LiquidTemplate ;
 // The port to allow 
      _port : LiquidTemplate ;
    constructor (
address:
 LiquidTemplate,
port:
 LiquidTemplate,
      ) {
          this._address = address;
          this._port = port;
    }

components(value: string[]) : UdpPortRestriction {
      this._components = value;
      return this;
    }
    getComponents() : string[] {
      return this._components;

    }
address(value: LiquidTemplate) : UdpPortRestriction {
      this._address = value;
      return this;
    }
    getAddress() : LiquidTemplate {
      return this._address;

    }
port(value: LiquidTemplate) : UdpPortRestriction {
      this._port = value;
      return this;
    }
    getPort() : LiquidTemplate {
      return this._port;

    }

    getKind() : string {
      return "wick/resource/udpport@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/resource/udpport@v1",
components: this._components,address: this._address,port: this._port,      }

    }
}

    
    
    
    



export class ComponentConfiguration implements HasKind {
 // The name of the component. 
      _name : string| undefined =  undefined;
 // Associated metadata for this component. 
      _metadata : Metadata| undefined =  undefined;
 // Details about the package for this component. 
      _package : PackageDefinition| undefined =  undefined;
 // Configuration for when wick hosts this component as a service. 
      _host : HostConfig| undefined =  undefined;
 // Resources that the component can access. 
      _resources : ResourceBinding[] =  [];
 // Components or types to import into this component&#x27;s scope. 
      _import : ImportBinding[] =  [];
 // Additional types to export and make available to the component. 
      _types : TypeDefinition[] =  [];
 // Interfaces the component requires to operate. 
      _requires : InterfaceBinding[] =  [];
 // Configuration specific to different kinds of components. 
      _component : ComponentKind ;
 // Assertions that can be run against the component to validate its behavior. 
      _tests : TestConfiguration[] =  [];
    constructor (
component:
 ComponentKind,
      ) {
          this._component = component;
    }

name(value: string| undefined) : ComponentConfiguration {
      this._name = value;
      return this;
    }
    getName() : string| undefined {
      return this._name;

    }
metadata(value: Metadata| undefined) : ComponentConfiguration {
      this._metadata = value;
      return this;
    }
    getMetadata() : Metadata| undefined {
      return this._metadata;

    }
package(value: PackageDefinition| undefined) : ComponentConfiguration {
      this._package = value;
      return this;
    }
    getPackage() : PackageDefinition| undefined {
      return this._package;

    }
host(value: HostConfig| undefined) : ComponentConfiguration {
      this._host = value;
      return this;
    }
    getHost() : HostConfig| undefined {
      return this._host;

    }
resources(value: ResourceBinding[]) : ComponentConfiguration {
      this._resources = value;
      return this;
    }
    getResources() : ResourceBinding[] {
      return this._resources;

    }
import(value: ImportBinding[]) : ComponentConfiguration {
      this._import = value;
      return this;
    }
    getImport() : ImportBinding[] {
      return this._import;

    }
types(value: TypeDefinition[]) : ComponentConfiguration {
      this._types = value;
      return this;
    }
    getTypes() : TypeDefinition[] {
      return this._types;

    }
requires(value: InterfaceBinding[]) : ComponentConfiguration {
      this._requires = value;
      return this;
    }
    getRequires() : InterfaceBinding[] {
      return this._requires;

    }
component(value: ComponentKind) : ComponentConfiguration {
      this._component = value;
      return this;
    }
    getComponent() : ComponentKind {
      return this._component;

    }
tests(value: TestConfiguration[]) : ComponentConfiguration {
      this._tests = value;
      return this;
    }
    getTests() : TestConfiguration[] {
      return this._tests;

    }

    getKind() : string {
      return "wick/component@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/component@v1",
name: this._name,metadata: this._metadata,package: this._package,host: this._host,resources: this._resources,import: this._import,types: this._types,requires: this._requires,component: this._component,tests: this._tests,      }

    }
}

    
    
    
    



export class InterfaceBinding implements HasKind {
 // The name of the interface. 
      _name : string ;
 // The interface to bind to. 
      _interface : InterfaceDefinition ;
    constructor (
name:
 string,
interface_:
 InterfaceDefinition,
      ) {
          this._name = name;
          this._interface = interface_;
    }

name(value: string) : InterfaceBinding {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
interface(value: InterfaceDefinition) : InterfaceBinding {
      this._interface = value;
      return this;
    }
    getInterface() : InterfaceDefinition {
      return this._interface;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,interface: this._interface,      }

    }
}

    
    
    
    



export class InterfaceDefinition implements HasKind {
 // Types used by the interface&#x27;s operations 
      _types : TypeDefinition[] =  [];
 // A list of operations defined by this interface. 
      _operations : OperationDefinition[] =  [];
    constructor (
      ) {
    }

types(value: TypeDefinition[]) : InterfaceDefinition {
      this._types = value;
      return this;
    }
    getTypes() : TypeDefinition[] {
      return this._types;

    }
operations(value: OperationDefinition[]) : InterfaceDefinition {
      this._operations = value;
      return this;
    }
    getOperations() : OperationDefinition[] {
      return this._operations;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
types: this._types,operations: this._operations,      }

    }
}

    
    
    
    



export class CompositeComponentConfiguration implements HasKind {
 // A list of operations exposed by the Composite component. 
      _operations : CompositeOperationDefinition[] =  [];
 // Configuration necessary to provide when instantiating the component. 
      _with : Field[] =  [];
 // A component or components whose operations you want to inherit from. 
      _extends : string[] =  [];
    constructor (
      ) {
    }

operations(value: CompositeOperationDefinition[]) : CompositeComponentConfiguration {
      this._operations = value;
      return this;
    }
    getOperations() : CompositeOperationDefinition[] {
      return this._operations;

    }
with(value: Field[]) : CompositeComponentConfiguration {
      this._with = value;
      return this;
    }
    getWith() : Field[] {
      return this._with;

    }
extends(value: string[]) : CompositeComponentConfiguration {
      this._extends = value;
      return this;
    }
    getExtends() : string[] {
      return this._extends;

    }

    getKind() : string {
      return "wick/component/composite@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/component/composite@v1",
operations: this._operations,with: this._with,extends: this._extends,      }

    }
}

    
    
    
    



export class WasmComponentConfiguration implements HasKind {
 // The path or OCI reference to the WebAssembly module 
      _ref : string ;
 // Volumes to expose to the component. 
      _volumes : ExposedVolume[] =  [];
 // The default size to allocate to the component&#x27;s send/receive buffer. 
      _maxPacketSize : number| undefined =  undefined;
 // Configuration necessary to provide when instantiating the component. 
      _with : Field[] =  [];
 // A list of operations implemented by the WebAssembly module. 
      _operations : OperationDefinition[] =  [];
    constructor (
ref:
 string,
      ) {
          this._ref = ref;
    }

ref(value: string) : WasmComponentConfiguration {
      this._ref = value;
      return this;
    }
    getRef() : string {
      return this._ref;

    }
volumes(value: ExposedVolume[]) : WasmComponentConfiguration {
      this._volumes = value;
      return this;
    }
    getVolumes() : ExposedVolume[] {
      return this._volumes;

    }
maxPacketSize(value: number| undefined) : WasmComponentConfiguration {
      this._maxPacketSize = value;
      return this;
    }
    getMaxPacketSize() : number| undefined {
      return this._maxPacketSize;

    }
with(value: Field[]) : WasmComponentConfiguration {
      this._with = value;
      return this;
    }
    getWith() : Field[] {
      return this._with;

    }
operations(value: OperationDefinition[]) : WasmComponentConfiguration {
      this._operations = value;
      return this;
    }
    getOperations() : OperationDefinition[] {
      return this._operations;

    }

    getKind() : string {
      return "wick/component/wasmrs@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/component/wasmrs@v1",
ref: this._ref,volumes: this._volumes,max_packet_size: this._maxPacketSize,with: this._with,operations: this._operations,      }

    }
}

    
    
    
    



export class WasmComponentModel implements HasKind {
 // The path or OCI reference to the WebAssembly module 
      _ref : string ;
 // Volumes to expose to the component. 
      _volumes : ExposedVolume[] =  [];
 // Configuration necessary to provide when instantiating the component. 
      _with : Field[] =  [];
 // A list of operations implemented by the WebAssembly module. 
      _operations : OperationDefinition[] =  [];
    constructor (
ref:
 string,
      ) {
          this._ref = ref;
    }

ref(value: string) : WasmComponentModel {
      this._ref = value;
      return this;
    }
    getRef() : string {
      return this._ref;

    }
volumes(value: ExposedVolume[]) : WasmComponentModel {
      this._volumes = value;
      return this;
    }
    getVolumes() : ExposedVolume[] {
      return this._volumes;

    }
with(value: Field[]) : WasmComponentModel {
      this._with = value;
      return this;
    }
    getWith() : Field[] {
      return this._with;

    }
operations(value: OperationDefinition[]) : WasmComponentModel {
      this._operations = value;
      return this;
    }
    getOperations() : OperationDefinition[] {
      return this._operations;

    }

    getKind() : string {
      return "wick/component/wasm@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/component/wasm@v1",
ref: this._ref,volumes: this._volumes,with: this._with,operations: this._operations,      }

    }
}

    
    
    
    



export class ExposedVolume implements HasKind {
 // The resource ID of the volume. 
      _resource : BoundIdentifier ;
 // The path to map it to in the component. 
      _path : string ;
    constructor (
resource:
 BoundIdentifier,
path:
 string,
      ) {
          this._resource = resource;
          this._path = path;
    }

resource(value: BoundIdentifier) : ExposedVolume {
      this._resource = value;
      return this;
    }
    getResource() : BoundIdentifier {
      return this._resource;

    }
path(value: string) : ExposedVolume {
      this._path = value;
      return this;
    }
    getPath() : string {
      return this._path;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
resource: this._resource,path: this._path,      }

    }
}

    
    
    
    

    
    
    
export type ComponentKind =
      WasmComponentConfiguration|WasmComponentModel|CompositeComponentConfiguration|SqlComponent|HttpClientComponent
    ;
    

    
    
    
export type ImportDefinition =
      TypesComponent|ManifestComponent|SqlComponent|HttpClientComponent
    ;
    

    
    
    
export type ComponentDefinition =
      GrpcUrlComponent|ManifestComponent|ComponentReference|SqlComponent|HttpClientComponent
    ;
    



export class TypesComponent implements HasKind {
 // The URL (and optional tag) or local file path to find the types manifest. 
      _ref : string ;
 // The types to import from the manifest. 
      _types : string[] =  [];
    constructor (
ref:
 string,
      ) {
          this._ref = ref;
    }

ref(value: string) : TypesComponent {
      this._ref = value;
      return this;
    }
    getRef() : string {
      return this._ref;

    }
types(value: string[]) : TypesComponent {
      this._types = value;
      return this;
    }
    getTypes() : string[] {
      return this._types;

    }

    getKind() : string {
      return "wick/component/types@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/component/types@v1",
ref: this._ref,types: this._types,      }

    }
}

    
    
    
    



export class ComponentReference implements HasKind {
 // The id of the referenced component. 
      _id : BoundIdentifier ;
    constructor (
id:
 BoundIdentifier,
      ) {
          this._id = id;
    }

id(value: BoundIdentifier) : ComponentReference {
      this._id = value;
      return this;
    }
    getId() : BoundIdentifier {
      return this._id;

    }

    getKind() : string {
      return "wick/component/reference@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/component/reference@v1",
id: this._id,      }

    }
}

    
    
    
    



export class HostConfig implements HasKind {
 // Whether or not to allow the &#x60;:latest&#x60; tag on remote artifacts. 
      _allowLatest : boolean =false;
 // A list of registries to connect to insecurely (over HTTP vs HTTPS). 
      _insecureRegistries : string[] =  [];
 // Configuration for the GRPC server. 
      _rpc : HttpConfig| undefined =  undefined;
    constructor (
      ) {
    }

allowLatest(value: boolean) : HostConfig {
      this._allowLatest = value;
      return this;
    }
    getAllowLatest() : boolean {
      return this._allowLatest;

    }
insecureRegistries(value: string[]) : HostConfig {
      this._insecureRegistries = value;
      return this;
    }
    getInsecureRegistries() : string[] {
      return this._insecureRegistries;

    }
rpc(value: HttpConfig| undefined) : HostConfig {
      this._rpc = value;
      return this;
    }
    getRpc() : HttpConfig| undefined {
      return this._rpc;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
allow_latest: this._allowLatest,insecure_registries: this._insecureRegistries,rpc: this._rpc,      }

    }
}

    
    
    
    



export class HttpConfig implements HasKind {
 // Enable/disable the server. 
      _enabled : boolean =false;
 // The port to bind to. 
      _port : number| undefined =  undefined;
 // The address to bind to. 
      _address : string| undefined =  undefined;
 // Path to pem file for TLS. 
      _pem : string| undefined =  undefined;
 // Path to key file for TLS. 
      _key : string| undefined =  undefined;
 // Path to CA file. 
      _ca : string| undefined =  undefined;
    constructor (
      ) {
    }

enabled(value: boolean) : HttpConfig {
      this._enabled = value;
      return this;
    }
    getEnabled() : boolean {
      return this._enabled;

    }
port(value: number| undefined) : HttpConfig {
      this._port = value;
      return this;
    }
    getPort() : number| undefined {
      return this._port;

    }
address(value: string| undefined) : HttpConfig {
      this._address = value;
      return this;
    }
    getAddress() : string| undefined {
      return this._address;

    }
pem(value: string| undefined) : HttpConfig {
      this._pem = value;
      return this;
    }
    getPem() : string| undefined {
      return this._pem;

    }
key(value: string| undefined) : HttpConfig {
      this._key = value;
      return this;
    }
    getKey() : string| undefined {
      return this._key;

    }
ca(value: string| undefined) : HttpConfig {
      this._ca = value;
      return this;
    }
    getCa() : string| undefined {
      return this._ca;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
enabled: this._enabled,port: this._port,address: this._address,pem: this._pem,key: this._key,ca: this._ca,      }

    }
}

    
    
    
    



export class GrpcUrlComponent implements HasKind {
 // The GRPC URL to connect to. 
      _url : string ;
 // Any configuration necessary for the component. 
      _with :   Record<string,LiquidJsonValue>| undefined =  undefined;
    constructor (
url:
 string,
      ) {
          this._url = url;
    }

url(value: string) : GrpcUrlComponent {
      this._url = value;
      return this;
    }
    getUrl() : string {
      return this._url;

    }
with(value:   Record<string,LiquidJsonValue>| undefined) : GrpcUrlComponent {
      this._with = value;
      return this;
    }
    getWith() :   Record<string,LiquidJsonValue>| undefined {
      return this._with;

    }

    getKind() : string {
      return "wick/component/grpc@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/component/grpc@v1",
url: this._url,with: this._with,      }

    }
}

    
    
    
    



export class ManifestComponent implements HasKind {
 // The URL (and optional tag) or local file path to find the manifest. 
      _ref : string ;
 // Any configuration necessary for the component. 
      _with :   Record<string,LiquidJsonValue>| undefined =  undefined;
 // External components to provide to the referenced component. 
      _provide :   Record<string,string> =  {};
 // If applicable, the default size to allocate to the component&#x27;s send/receive buffer. 
      _maxPacketSize : number| undefined =  undefined;
    constructor (
ref:
 string,
      ) {
          this._ref = ref;
    }

ref(value: string) : ManifestComponent {
      this._ref = value;
      return this;
    }
    getRef() : string {
      return this._ref;

    }
with(value:   Record<string,LiquidJsonValue>| undefined) : ManifestComponent {
      this._with = value;
      return this;
    }
    getWith() :   Record<string,LiquidJsonValue>| undefined {
      return this._with;

    }
provide(value:   Record<string,string>) : ManifestComponent {
      this._provide = value;
      return this;
    }
    getProvide() :   Record<string,string> {
      return this._provide;

    }
maxPacketSize(value: number| undefined) : ManifestComponent {
      this._maxPacketSize = value;
      return this;
    }
    getMaxPacketSize() : number| undefined {
      return this._maxPacketSize;

    }

    getKind() : string {
      return "wick/component/manifest@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/component/manifest@v1",
ref: this._ref,with: this._with,provide: this._provide,max_packet_size: this._maxPacketSize,      }

    }
}

    
    
    
    



export class CompositeOperationDefinition implements HasKind {
 // The name of the operation. 
      _name : string ="";
 // Any configuration required by the operation. 
      _with : Field[] =  [];
 // Types of the inputs to the operation. 
      _inputs : Field[] =  [];
 // Types of the outputs to the operation. 
      _outputs : Field[] =  [];
 // A map of IDs to specific operations. 
      _uses : OperationInstance[] =  [];
 // A list of connections from operation to operation. 
      _flow : FlowExpression[] =  [];
 // Additional &#x60;CompositeOperationDefinition&#x60;s to define as children. 
      _operations : CompositeOperationDefinition[] =  [];
    constructor (
      ) {
    }

name(value: string) : CompositeOperationDefinition {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
with(value: Field[]) : CompositeOperationDefinition {
      this._with = value;
      return this;
    }
    getWith() : Field[] {
      return this._with;

    }
inputs(value: Field[]) : CompositeOperationDefinition {
      this._inputs = value;
      return this;
    }
    getInputs() : Field[] {
      return this._inputs;

    }
outputs(value: Field[]) : CompositeOperationDefinition {
      this._outputs = value;
      return this;
    }
    getOutputs() : Field[] {
      return this._outputs;

    }
uses(value: OperationInstance[]) : CompositeOperationDefinition {
      this._uses = value;
      return this;
    }
    getUses() : OperationInstance[] {
      return this._uses;

    }
flow(value: FlowExpression[]) : CompositeOperationDefinition {
      this._flow = value;
      return this;
    }
    getFlow() : FlowExpression[] {
      return this._flow;

    }
operations(value: CompositeOperationDefinition[]) : CompositeOperationDefinition {
      this._operations = value;
      return this;
    }
    getOperations() : CompositeOperationDefinition[] {
      return this._operations;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,with: this._with,inputs: this._inputs,outputs: this._outputs,uses: this._uses,flow: this._flow,operations: this._operations,      }

    }
}

    
    
    
    

    
    
    
export type FlowExpression =
string |      ConnectionDefinition|BlockExpression
    ;
    



export class BlockExpression implements HasKind {

      _expressions : FlowExpression[] ;
    constructor (
expressions:
 FlowExpression[],
      ) {
          this._expressions = expressions;
    }

expressions(value: FlowExpression[]) : BlockExpression {
      this._expressions = value;
      return this;
    }
    getExpressions() : FlowExpression[] {
      return this._expressions;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
expressions: this._expressions,      }

    }
}

    
    
    
    



export class ConnectionDefinition implements HasKind {
 // An upstream operation&#x27;s output. 
      _from : ConnectionTargetDefinition ;
 // A downstream operation&#x27;s input. 
      _to : ConnectionTargetDefinition ;
    constructor (
from:
 ConnectionTargetDefinition,
to:
 ConnectionTargetDefinition,
      ) {
          this._from = from;
          this._to = to;
    }

from(value: ConnectionTargetDefinition) : ConnectionDefinition {
      this._from = value;
      return this;
    }
    getFrom() : ConnectionTargetDefinition {
      return this._from;

    }
to(value: ConnectionTargetDefinition) : ConnectionDefinition {
      this._to = value;
      return this;
    }
    getTo() : ConnectionTargetDefinition {
      return this._to;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
from: this._from,to: this._to,      }

    }
}

    
    
    
    



export class ConnectionTargetDefinition implements HasKind {
 // The instance ID of the component operation. 
      _instance : string ;
 // The operation&#x27;s input or output (depending on to/from). 
      _port : string| undefined =  undefined;
 // The default value to provide on this connection in the event of an error. 
      _data :   Record<string,LiquidJsonValue>| undefined =  undefined;
    constructor (
instance:
 string,
      ) {
          this._instance = instance;
    }

instance(value: string) : ConnectionTargetDefinition {
      this._instance = value;
      return this;
    }
    getInstance() : string {
      return this._instance;

    }
port(value: string| undefined) : ConnectionTargetDefinition {
      this._port = value;
      return this;
    }
    getPort() : string| undefined {
      return this._port;

    }
data(value:   Record<string,LiquidJsonValue>| undefined) : ConnectionTargetDefinition {
      this._data = value;
      return this;
    }
    getData() :   Record<string,LiquidJsonValue>| undefined {
      return this._data;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
instance: this._instance,port: this._port,data: this._data,      }

    }
}

    
    
    
    



export class OperationDefinition implements HasKind {
 // The name of the operation. 
      _name : string ="";
 // Any configuration required by the operation. 
      _with : Field[] =  [];
 // Types of the inputs to the operation. 
      _inputs : Field[] =  [];
 // Types of the outputs to the operation. 
      _outputs : Field[] =  [];
    constructor (
      ) {
    }

name(value: string) : OperationDefinition {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
with(value: Field[]) : OperationDefinition {
      this._with = value;
      return this;
    }
    getWith() : Field[] {
      return this._with;

    }
inputs(value: Field[]) : OperationDefinition {
      this._inputs = value;
      return this;
    }
    getInputs() : Field[] {
      return this._inputs;

    }
outputs(value: Field[]) : OperationDefinition {
      this._outputs = value;
      return this;
    }
    getOutputs() : Field[] {
      return this._outputs;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,with: this._with,inputs: this._inputs,outputs: this._outputs,      }

    }
}

    
    
    
    



export class Field implements HasKind {
 // The name of the field. 
      _name : string ;
 // The type signature of the field. 
      _type : TypeSignature ;
 // The description of the field. 
      _description : string| undefined =  undefined;
    constructor (
name:
 string,
type_:
 TypeSignature,
      ) {
          this._name = name;
          this._type = type_;
    }

name(value: string) : Field {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
type(value: TypeSignature) : Field {
      this._type = value;
      return this;
    }
    getType() : TypeSignature {
      return this._type;

    }
description(value: string| undefined) : Field {
      this._description = value;
      return this;
    }
    getDescription() : string| undefined {
      return this._description;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,type: this._type,description: this._description,      }

    }
}

    
    
    
    

    
    
    
export type TypeSignature =
      string
;
    



export const I8 = "i8";

    
    
    
    



export const I16 = "i16";

    
    
    
    



export const I32 = "i32";

    
    
    
    



export const I64 = "i64";

    
    
    
    



export const U8 = "u8";

    
    
    
    



export const U16 = "u16";

    
    
    
    



export const U32 = "u32";

    
    
    
    



export const U64 = "u64";

    
    
    
    



export const F32 = "f32";

    
    
    
    



export const F64 = "f64";

    
    
    
    



export const Bool = "i8";

    
    
    
    



export const StringType = "string";

    
    
    
    



export const Datetime = "datetime";

    
    
    
    



export const Bytes = "bytes";

    
    
    
    



export class Custom implements HasKind {
 // The name of the custom type. 
      _name : string ="";
    constructor (
      ) {
    }

name(value: string) : Custom {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,      }

    }
}

    
    
    
    



export class Optional implements HasKind {

      _type : TypeSignature ;
    constructor (
type_:
 TypeSignature,
      ) {
          this._type = type_;
    }

type(value: TypeSignature) : Optional {
      this._type = value;
      return this;
    }
    getType() : TypeSignature {
      return this._type;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
type: this._type,      }

    }
}

    
    
    
    



export class List implements HasKind {

      _type : TypeSignature ;
    constructor (
type_:
 TypeSignature,
      ) {
          this._type = type_;
    }

type(value: TypeSignature) : List {
      this._type = value;
      return this;
    }
    getType() : TypeSignature {
      return this._type;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
type: this._type,      }

    }
}

    
    
    
    



export class Map implements HasKind {

      _key : TypeSignature ;

      _value : TypeSignature ;
    constructor (
key:
 TypeSignature,
value:
 TypeSignature,
      ) {
          this._key = key;
          this._value = value;
    }

key(value: TypeSignature) : Map {
      this._key = value;
      return this;
    }
    getKey() : TypeSignature {
      return this._key;

    }
value(value: TypeSignature) : Map {
      this._value = value;
      return this;
    }
    getValue() : TypeSignature {
      return this._value;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
key: this._key,value: this._value,      }

    }
}

    
    
    
    



export const Object = "object";

    
    
    
    

    
    
    
export type TypeDefinition =
      StructSignature|EnumSignature|UnionSignature
    ;
    



export class StructSignature implements HasKind {
 // The name of the struct. 
      _name : string ="";
 // The fields in this struct. 
      _fields : Field[] =  [];
 // The description of the struct. 
      _description : string| undefined =  undefined;
    constructor (
      ) {
    }

name(value: string) : StructSignature {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
fields(value: Field[]) : StructSignature {
      this._fields = value;
      return this;
    }
    getFields() : Field[] {
      return this._fields;

    }
description(value: string| undefined) : StructSignature {
      this._description = value;
      return this;
    }
    getDescription() : string| undefined {
      return this._description;

    }

    getKind() : string {
      return "wick/type/struct@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/type/struct@v1",
name: this._name,fields: this._fields,description: this._description,      }

    }
}

    
    
    
    



export class UnionSignature implements HasKind {
 // The name of the union. 
      _name : string ="";
 // The types in the union. 
      _types : TypeSignature[] =  [];
 // The description of the union. 
      _description : string| undefined =  undefined;
    constructor (
      ) {
    }

name(value: string) : UnionSignature {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
types(value: TypeSignature[]) : UnionSignature {
      this._types = value;
      return this;
    }
    getTypes() : TypeSignature[] {
      return this._types;

    }
description(value: string| undefined) : UnionSignature {
      this._description = value;
      return this;
    }
    getDescription() : string| undefined {
      return this._description;

    }

    getKind() : string {
      return "wick/type/union@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/type/union@v1",
name: this._name,types: this._types,description: this._description,      }

    }
}

    
    
    
    



export class EnumSignature implements HasKind {
 // The name of the enum. 
      _name : string ="";
 // The variants in the enum. 
      _variants : EnumVariant[] =  [];
 // The description of the enum. 
      _description : string| undefined =  undefined;
    constructor (
      ) {
    }

name(value: string) : EnumSignature {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
variants(value: EnumVariant[]) : EnumSignature {
      this._variants = value;
      return this;
    }
    getVariants() : EnumVariant[] {
      return this._variants;

    }
description(value: string| undefined) : EnumSignature {
      this._description = value;
      return this;
    }
    getDescription() : string| undefined {
      return this._description;

    }

    getKind() : string {
      return "wick/type/enum@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/type/enum@v1",
name: this._name,variants: this._variants,description: this._description,      }

    }
}

    
    
    
    



export class EnumVariant implements HasKind {
 // The name of the variant. 
      _name : string ="";
 // The index of the variant. 
      _index : number| undefined =  undefined;
 // The optional value of the variant. 
      _value : string| undefined =  undefined;
 // A description of the variant. 
      _description : string| undefined =  undefined;
    constructor (
      ) {
    }

name(value: string) : EnumVariant {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
index(value: number| undefined) : EnumVariant {
      this._index = value;
      return this;
    }
    getIndex() : number| undefined {
      return this._index;

    }
value(value: string| undefined) : EnumVariant {
      this._value = value;
      return this;
    }
    getValue() : string| undefined {
      return this._value;

    }
description(value: string| undefined) : EnumVariant {
      this._description = value;
      return this;
    }
    getDescription() : string| undefined {
      return this._description;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,index: this._index,value: this._value,description: this._description,      }

    }
}

    
    
    
    



export class OperationInstance implements HasKind {
 // The name of the binding. 
      _name : string ;
 // The operation to bind to. 
      _operation :string | ComponentOperationExpression ;
 // Data to associate with the reference, if any. 
      _with :   Record<string,LiquidJsonValue>| undefined =  undefined;
 // Timeout (in milliseconds) to wait for the operation to complete. Use 0 to wait indefinitely. 
      _timeout : number| undefined =  undefined;
    constructor (
name:
 string,
operation:
string | ComponentOperationExpression,
      ) {
          this._name = name;
          this._operation = operation;
    }

name(value: string) : OperationInstance {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
operation(value: ComponentOperationExpression) : OperationInstance {
      this._operation = value;
      return this;
    }
    getOperation() :string | ComponentOperationExpression {
      return this._operation;

    }
with(value:   Record<string,LiquidJsonValue>| undefined) : OperationInstance {
      this._with = value;
      return this;
    }
    getWith() :   Record<string,LiquidJsonValue>| undefined {
      return this._with;

    }
timeout(value: number| undefined) : OperationInstance {
      this._timeout = value;
      return this;
    }
    getTimeout() : number| undefined {
      return this._timeout;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,operation: this._operation,with: this._with,timeout: this._timeout,      }

    }
}

    
    
    
    



export class TestDefinition implements HasKind {
 // The name of the test. 
      _name : string| undefined =  undefined;
 // The operaton to test. 
      _operation : string ;
 // Inherent data to use for the test. 
      _inherent : InherentData| undefined =  undefined;
 // The configuration for the operation, if any. 
      _with :   Record<string,LiquidJsonValue>| undefined =  undefined;
 // The inputs to the test. 
      _inputs : PacketData[] =  [];
 // The expected outputs of the operation. 
      _outputs : TestPacketData[] =  [];
    constructor (
operation:
 string,
      ) {
          this._operation = operation;
    }

name(value: string| undefined) : TestDefinition {
      this._name = value;
      return this;
    }
    getName() : string| undefined {
      return this._name;

    }
operation(value: string) : TestDefinition {
      this._operation = value;
      return this;
    }
    getOperation() : string {
      return this._operation;

    }
inherent(value: InherentData| undefined) : TestDefinition {
      this._inherent = value;
      return this;
    }
    getInherent() : InherentData| undefined {
      return this._inherent;

    }
with(value:   Record<string,LiquidJsonValue>| undefined) : TestDefinition {
      this._with = value;
      return this;
    }
    getWith() :   Record<string,LiquidJsonValue>| undefined {
      return this._with;

    }
inputs(value: PacketData[]) : TestDefinition {
      this._inputs = value;
      return this;
    }
    getInputs() : PacketData[] {
      return this._inputs;

    }
outputs(value: TestPacketData[]) : TestDefinition {
      this._outputs = value;
      return this;
    }
    getOutputs() : TestPacketData[] {
      return this._outputs;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,operation: this._operation,inherent: this._inherent,with: this._with,inputs: this._inputs,outputs: this._outputs,      }

    }
}

    
    
    
    



export class InherentData implements HasKind {
 // A random seed, i.e. to initialize a random number generator. 
      _seed : number| undefined =  undefined;
 // A timestamp. 
      _timestamp : number| undefined =  undefined;
    constructor (
      ) {
    }

seed(value: number| undefined) : InherentData {
      this._seed = value;
      return this;
    }
    getSeed() : number| undefined {
      return this._seed;

    }
timestamp(value: number| undefined) : InherentData {
      this._timestamp = value;
      return this;
    }
    getTimestamp() : number| undefined {
      return this._timestamp;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
seed: this._seed,timestamp: this._timestamp,      }

    }
}

    
    
    
    

    
    
    
export type PacketData =
      SuccessPacket|SignalPacket|ErrorPacket
    ;
    

    
    
    
export type TestPacketData =
      SuccessPacket|SignalPacket|PacketAssertionDef|ErrorPacket
    ;
    



export class SignalPacket implements HasKind {
 // The name of the input or output this packet is going to or coming from. 
      _name : string ;
 // Any flags set on the packet. Deprecated, use &#x27;flag:&#x27; instead 
      _flags : PacketFlags| undefined =  undefined;
 // The flag set on the packet. 
      _flag : PacketFlag| undefined =  undefined;
    constructor (
name:
 string,
      ) {
          this._name = name;
    }

name(value: string) : SignalPacket {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
flags(value: PacketFlags| undefined) : SignalPacket {
      this._flags = value;
      return this;
    }
    getFlags() : PacketFlags| undefined {
      return this._flags;

    }
flag(value: PacketFlag| undefined) : SignalPacket {
      this._flag = value;
      return this;
    }
    getFlag() : PacketFlag| undefined {
      return this._flag;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,flags: this._flags,flag: this._flag,      }

    }
}

    
    
    
    



export class SuccessPacket implements HasKind {
 // The name of the input or output this packet is going to or coming from. 
      _name : string ;
 // The packet payload. 
      _value : LiquidJsonValue ;
    constructor (
name:
 string,
value:
 LiquidJsonValue,
      ) {
          this._name = name;
          this._value = value;
    }

name(value: string) : SuccessPacket {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
value(value: LiquidJsonValue) : SuccessPacket {
      this._value = value;
      return this;
    }
    getValue() : LiquidJsonValue {
      return this._value;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,value: this._value,      }

    }
}

    
    
    
    



export class PacketAssertionDef implements HasKind {
 // The name of the input or output this packet is going to or coming from. 
      _name : string ;
 // An assertion to test against the packet. 
      _assertions : PacketAssertion[] =  [];
    constructor (
name:
 string,
      ) {
          this._name = name;
    }

name(value: string) : PacketAssertionDef {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
assertions(value: PacketAssertion[]) : PacketAssertionDef {
      this._assertions = value;
      return this;
    }
    getAssertions() : PacketAssertion[] {
      return this._assertions;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,assertions: this._assertions,      }

    }
}

    
    
    
    



export class PacketAssertion implements HasKind {
 // The optional path to a value in the packet to assert against. 
      _path : string| undefined =  undefined;
 // The operation to use when asserting against a packet. 
      _operator : AssertionOperator ;
 // A value or object combine with the operator to assert against a packet value. 
      _value : LiquidJsonValue ;
    constructor (
operator:
 AssertionOperator,
value:
 LiquidJsonValue,
      ) {
          this._operator = operator;
          this._value = value;
    }

path(value: string| undefined) : PacketAssertion {
      this._path = value;
      return this;
    }
    getPath() : string| undefined {
      return this._path;

    }
operator(value: AssertionOperator) : PacketAssertion {
      this._operator = value;
      return this;
    }
    getOperator() : AssertionOperator {
      return this._operator;

    }
value(value: LiquidJsonValue) : PacketAssertion {
      this._value = value;
      return this;
    }
    getValue() : LiquidJsonValue {
      return this._value;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
path: this._path,operator: this._operator,value: this._value,      }

    }
}

    
    
    
    

    
    
export enum AssertionOperator {
Equals = "Equals",LessThan = "LessThan",GreaterThan = "GreaterThan",Regex = "Regex",Contains = "Contains",}
    
    



export class ErrorPacket implements HasKind {
 // The name of the input or output this packet is going to or coming from. 
      _name : string ;
 // Any flags set on the packet. Deprecated, use &#x27;flag:&#x27; instead 
      _flags : PacketFlags| undefined =  undefined;
 // The flag set on the packet. 
      _flag : PacketFlag| undefined =  undefined;
 // The error message. 
      _error : LiquidTemplate ;
    constructor (
name:
 string,
error:
 LiquidTemplate,
      ) {
          this._name = name;
          this._error = error;
    }

name(value: string) : ErrorPacket {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
flags(value: PacketFlags| undefined) : ErrorPacket {
      this._flags = value;
      return this;
    }
    getFlags() : PacketFlags| undefined {
      return this._flags;

    }
flag(value: PacketFlag| undefined) : ErrorPacket {
      this._flag = value;
      return this;
    }
    getFlag() : PacketFlag| undefined {
      return this._flag;

    }
error(value: LiquidTemplate) : ErrorPacket {
      this._error = value;
      return this;
    }
    getError() : LiquidTemplate {
      return this._error;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,flags: this._flags,flag: this._flag,error: this._error,      }

    }
}

    
    
    
    



export class PacketFlags implements HasKind {
 // Indicates the port should be considered closed. 
      _done : boolean =false;
 // Indicates the opening of a new substream context within the parent stream. 
      _open : boolean =false;
 // Indicates the closing of a substream context within the parent stream. 
      _close : boolean =false;
    constructor (
      ) {
    }

done(value: boolean) : PacketFlags {
      this._done = value;
      return this;
    }
    getDone() : boolean {
      return this._done;

    }
open(value: boolean) : PacketFlags {
      this._open = value;
      return this;
    }
    getOpen() : boolean {
      return this._open;

    }
close(value: boolean) : PacketFlags {
      this._close = value;
      return this;
    }
    getClose() : boolean {
      return this._close;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
done: this._done,open: this._open,close: this._close,      }

    }
}

    
    
    
    

    
    
export enum PacketFlag {
Done = "Done",Open = "Open",Close = "Close",}
    
    



export class SqlComponent implements HasKind {
 // The connect string URL resource for the database. 
      _resource : BoundIdentifier ;
 // Whether or not to use TLS. 
      _tls : boolean =false;
 // Configuration necessary to provide when instantiating the component. 
      _with : Field[] =  [];
 // A list of operations to expose on this component. 
      _operations : SqlQueryKind[] =  [];
    constructor (
resource:
 BoundIdentifier,
      ) {
          this._resource = resource;
    }

resource(value: BoundIdentifier) : SqlComponent {
      this._resource = value;
      return this;
    }
    getResource() : BoundIdentifier {
      return this._resource;

    }
tls(value: boolean) : SqlComponent {
      this._tls = value;
      return this;
    }
    getTls() : boolean {
      return this._tls;

    }
with(value: Field[]) : SqlComponent {
      this._with = value;
      return this;
    }
    getWith() : Field[] {
      return this._with;

    }
operations(value: SqlQueryKind[]) : SqlComponent {
      this._operations = value;
      return this;
    }
    getOperations() : SqlQueryKind[] {
      return this._operations;

    }

    getKind() : string {
      return "wick/component/sql@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/component/sql@v1",
resource: this._resource,tls: this._tls,with: this._with,operations: this._operations,      }

    }
}

    
    
    
    

    
    
    
export type SqlQueryKind =
      SqlQueryOperationDefinition|SqlExecOperationDefinition
    ;
    



export class SqlQueryOperationDefinition implements HasKind {
 // The name of the operation. 
      _name : string ;
 // Any configuration required by the operation. 
      _with : Field[] =  [];
 // Types of the inputs to the operation. 
      _inputs : Field[] =  [];
 // Types of the outputs to the operation. 
      _outputs : Field[] =  [];
 // The query to execute. 
      _query : string ;
 // The positional arguments to the query, defined as a list of input names. 
      _arguments : string[] =  [];
 // What to do when an error occurs. 
      _onError : ErrorBehavior| undefined =  undefined;
    constructor (
name:
 string,
query:
 string,
      ) {
          this._name = name;
          this._query = query;
    }

name(value: string) : SqlQueryOperationDefinition {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
with(value: Field[]) : SqlQueryOperationDefinition {
      this._with = value;
      return this;
    }
    getWith() : Field[] {
      return this._with;

    }
inputs(value: Field[]) : SqlQueryOperationDefinition {
      this._inputs = value;
      return this;
    }
    getInputs() : Field[] {
      return this._inputs;

    }
outputs(value: Field[]) : SqlQueryOperationDefinition {
      this._outputs = value;
      return this;
    }
    getOutputs() : Field[] {
      return this._outputs;

    }
query(value: string) : SqlQueryOperationDefinition {
      this._query = value;
      return this;
    }
    getQuery() : string {
      return this._query;

    }
arguments(value: string[]) : SqlQueryOperationDefinition {
      this._arguments = value;
      return this;
    }
    getArguments() : string[] {
      return this._arguments;

    }
onError(value: ErrorBehavior| undefined) : SqlQueryOperationDefinition {
      this._onError = value;
      return this;
    }
    getOnError() : ErrorBehavior| undefined {
      return this._onError;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,with: this._with,inputs: this._inputs,outputs: this._outputs,query: this._query,arguments: this._arguments,on_error: this._onError,      }

    }
}

    
    
    
    



export class SqlExecOperationDefinition implements HasKind {
 // The name of the operation. 
      _name : string ;
 // Any configuration required by the operation. 
      _with : Field[] =  [];
 // Types of the inputs to the operation. 
      _inputs : Field[] =  [];
 // Types of the outputs to the operation. 
      _outputs : Field[] =  [];
 // The query to execute. 
      _exec : string ;
 // The positional arguments to the query, defined as a list of input names. 
      _arguments : string[] =  [];
 // What to do when an error occurs. 
      _onError : ErrorBehavior| undefined =  undefined;
    constructor (
name:
 string,
exec:
 string,
      ) {
          this._name = name;
          this._exec = exec;
    }

name(value: string) : SqlExecOperationDefinition {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
with(value: Field[]) : SqlExecOperationDefinition {
      this._with = value;
      return this;
    }
    getWith() : Field[] {
      return this._with;

    }
inputs(value: Field[]) : SqlExecOperationDefinition {
      this._inputs = value;
      return this;
    }
    getInputs() : Field[] {
      return this._inputs;

    }
outputs(value: Field[]) : SqlExecOperationDefinition {
      this._outputs = value;
      return this;
    }
    getOutputs() : Field[] {
      return this._outputs;

    }
exec(value: string) : SqlExecOperationDefinition {
      this._exec = value;
      return this;
    }
    getExec() : string {
      return this._exec;

    }
arguments(value: string[]) : SqlExecOperationDefinition {
      this._arguments = value;
      return this;
    }
    getArguments() : string[] {
      return this._arguments;

    }
onError(value: ErrorBehavior| undefined) : SqlExecOperationDefinition {
      this._onError = value;
      return this;
    }
    getOnError() : ErrorBehavior| undefined {
      return this._onError;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,with: this._with,inputs: this._inputs,outputs: this._outputs,exec: this._exec,arguments: this._arguments,on_error: this._onError,      }

    }
}

    
    
    
    

    
    
export enum ErrorBehavior {
Ignore = "Ignore",Commit = "Commit",Rollback = "Rollback",}
    
    



export class HttpClientComponent implements HasKind {
 // The URL base to use. 
      _resource : BoundIdentifier ;
 // The codec to use when encoding/decoding data. Can be overridden by individual operations. 
      _codec : Codec| undefined =  undefined;
 // The proxy HTTP / HTTPS to use. 
      _proxy : Proxy| undefined =  undefined;
 // The timeout in seconds 
      _timeout : number| undefined =  undefined;
 // Configuration necessary to provide when instantiating the component. 
      _with : Field[] =  [];
 // A list of operations to expose on this component. 
      _operations : HttpClientOperationDefinition[] =  [];
    constructor (
resource:
 BoundIdentifier,
      ) {
          this._resource = resource;
    }

resource(value: BoundIdentifier) : HttpClientComponent {
      this._resource = value;
      return this;
    }
    getResource() : BoundIdentifier {
      return this._resource;

    }
codec(value: Codec| undefined) : HttpClientComponent {
      this._codec = value;
      return this;
    }
    getCodec() : Codec| undefined {
      return this._codec;

    }
proxy(value: Proxy| undefined) : HttpClientComponent {
      this._proxy = value;
      return this;
    }
    getProxy() : Proxy| undefined {
      return this._proxy;

    }
timeout(value: number| undefined) : HttpClientComponent {
      this._timeout = value;
      return this;
    }
    getTimeout() : number| undefined {
      return this._timeout;

    }
with(value: Field[]) : HttpClientComponent {
      this._with = value;
      return this;
    }
    getWith() : Field[] {
      return this._with;

    }
operations(value: HttpClientOperationDefinition[]) : HttpClientComponent {
      this._operations = value;
      return this;
    }
    getOperations() : HttpClientOperationDefinition[] {
      return this._operations;

    }

    getKind() : string {
      return "wick/component/http@v1";
    }

    toJSON() : any {
      return {
        kind : "wick/component/http@v1",
resource: this._resource,codec: this._codec,proxy: this._proxy,timeout: this._timeout,with: this._with,operations: this._operations,      }

    }
}

    
    
    
    



export class Proxy implements HasKind {
 // The URL base to use. http, https are supported. 
      _resource : string ="";
 // The username to use when authenticating with the proxy. 
      _username : string| undefined =  undefined;
 // The password to use when authenticating with the proxy. 
      _password : string| undefined =  undefined;
    constructor (
      ) {
    }

resource(value: string) : Proxy {
      this._resource = value;
      return this;
    }
    getResource() : string {
      return this._resource;

    }
username(value: string| undefined) : Proxy {
      this._username = value;
      return this;
    }
    getUsername() : string| undefined {
      return this._username;

    }
password(value: string| undefined) : Proxy {
      this._password = value;
      return this;
    }
    getPassword() : string| undefined {
      return this._password;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
resource: this._resource,username: this._username,password: this._password,      }

    }
}

    
    
    
    



export class HttpClientOperationDefinition implements HasKind {
 // The name of the operation. 
      _name : string ;
 // Any configuration required by the operation. 
      _with : Field[] =  [];
 // Types of the inputs to the operation. 
      _inputs : Field[] =  [];
 // The HTTP method to use. 
      _method : HttpMethod ;
 // The codec to use when encoding/decoding data. 
      _codec : Codec| undefined =  undefined;
 // Any headers to add to the request. 
      _headers :   Record<string,string[]>| undefined =  undefined;
 // The body to send, processed as a structured JSON liquid template. 
      _body : LiquidJsonValue| undefined =  undefined;
 // The path to append to our base URL, processed as a liquid template with each input as part of the template data. 
      _path : string ="";
    constructor (
name:
 string,
method:
 HttpMethod,
      ) {
          this._name = name;
          this._method = method;
    }

name(value: string) : HttpClientOperationDefinition {
      this._name = value;
      return this;
    }
    getName() : string {
      return this._name;

    }
with(value: Field[]) : HttpClientOperationDefinition {
      this._with = value;
      return this;
    }
    getWith() : Field[] {
      return this._with;

    }
inputs(value: Field[]) : HttpClientOperationDefinition {
      this._inputs = value;
      return this;
    }
    getInputs() : Field[] {
      return this._inputs;

    }
method(value: HttpMethod) : HttpClientOperationDefinition {
      this._method = value;
      return this;
    }
    getMethod() : HttpMethod {
      return this._method;

    }
codec(value: Codec| undefined) : HttpClientOperationDefinition {
      this._codec = value;
      return this;
    }
    getCodec() : Codec| undefined {
      return this._codec;

    }
headers(value:   Record<string,string[]>| undefined) : HttpClientOperationDefinition {
      this._headers = value;
      return this;
    }
    getHeaders() :   Record<string,string[]>| undefined {
      return this._headers;

    }
body(value: LiquidJsonValue| undefined) : HttpClientOperationDefinition {
      this._body = value;
      return this;
    }
    getBody() : LiquidJsonValue| undefined {
      return this._body;

    }
path(value: string) : HttpClientOperationDefinition {
      this._path = value;
      return this;
    }
    getPath() : string {
      return this._path;

    }

    getKind() : string {
      return "";
    }

    toJSON() : any {
      return {
name: this._name,with: this._with,inputs: this._inputs,method: this._method,codec: this._codec,headers: this._headers,body: this._body,path: this._path,      }

    }
}

    
    
    
    

    
    
export enum Codec {
Json = "Json",Raw = "Raw",FormData = "FormData",Text = "Text",}
    
    

    
    
export enum HttpMethod {
Get = "Get",Post = "Post",Put = "Put",Delete = "Delete",}
    
    


