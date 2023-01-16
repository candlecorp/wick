import { BaseVisitor, Context, Kind } from "../../codegen/deps/model.ts";
import {
  AliasVisitor,
  EnumVisitor,
  expandType,
  mapArg,
  mapArgs,
} from "../../codegen/deps/typescript.ts";
import {
  camelCase,
  capitalize,
  convertOperationToType,
  formatComment,
} from "../../codegen/deps/utils.ts";

const defaultMod = new URL("./mod.ts", import.meta.url).toString();

export default class IotaTSVisitor extends BaseVisitor {
  public visitContextBefore(context: Context): void {
    const mod = context.config.mod || defaultMod;
    this.write(`// deno-lint-ignore-file no-unused-vars ban-unused-ignore
export * from "${mod}";
import {
  Application,
  Authorization,
  callInterface,
  callProvider,
  Flow,
  Handler,
  Entity,
  Response,
  toDataExpr,
  Operations,
  Step,
} from "${mod}";\n\n`);

    const types = new TypesVisitor(this.writer);
    context.accept(context, types);
  }

  visitInterfaceBefore(context: Context): void {
    const args = new ArgsVisitor(this.writer);
    context.interface.accept(context, args);

    const opers = new OpersVisitor(this.writer);
    context.interface.accept(context, opers);

    const auths = new AuthsVisitor(this.writer);
    context.interface.accept(context, auths);

    const iface = new InterfaceVisitor(this.writer);
    context.interface.accept(context, iface);

    const client = new ClientVisitor(this.writer);
    context.interface.accept(context, client);
  }

  visitAlias(context: Context): void {
    const e = new AliasVisitor(this.writer);
    context.alias.accept(context, e);
  }

  visitEnum(context: Context): void {
    const e = new EnumVisitor(this.writer);
    context.enum.accept(context, e);
  }

  visitType(context: Context): void {
    const type = new TypeVisitor(this.writer);
    context.type.accept(context, type);
  }
}

class TypeVisitor extends BaseVisitor {
  visitTypeBefore(context: Context): void {
    super.triggerTypeBefore(context);
    const t = context.type!;
    this.write(formatComment("// ", t.description));
    this.write(`export interface ${t.name} {\n`);
  }

  visitTypeField(context: Context): void {
    const field = context.field!;
    this.write(formatComment("  // ", field.description));
    const optional = field.type.kind == Kind.Optional ? "?" : "";
    const et = expandType(field.type!, false);
    this.write(`  ${field.name}${optional}: ${et};\n`);
    super.triggerTypeField(context);
  }

  visitTypeAfter(context: Context): void {
    this.write(`}\n\n`);
    super.triggerTypeAfter(context);
  }
}

class ArgsVisitor extends BaseVisitor {
  visitOperation(context: Context): void {
    const { interface: iface, operation } = context;
    if (operation.isUnary() || operation.parameters.length == 0) {
      return;
    }
    const args = convertOperationToType(context.getType, iface, operation);

    const types = new TypeVisitor(this.writer);
    args.accept(context.clone({ type: args }), types);
    super.triggerOperation(context);
  }
}

class TypesVisitor extends BaseVisitor {
  visitContextBefore(_context: Context): void {
    this.write("export const types = {\n");
  }

  visitContextAfter(_context: Context): void {
    this.write("}\n\n");
  }

  visitType(context: Context): void {
    const { namespace, type } = context;
    this.write(
      `  ${type.name}: "${namespace.name}::${type.name}" as Entity,\n`,
    );
  }

  visitEnum(context: Context): void {
    const { namespace, enum: e } = context;
    this.write(`  ${e.name}: "${namespace.name}::${e.name}" as Entity,\n`);
  }
}

class OpersVisitor extends BaseVisitor {
  visitInterfaceBefore(context: Context): void {
    super.triggerInterfaceBefore(context);
    const { interface: iface } = context;
    this.write(formatComment("// ", iface.description));
    this.write(`export interface ${iface.name}Oper {\n`);
  }

  visitOperation(context: Context): void {
    const { interface: iface, operation } = context;
    const optionalSuffix = iface.annotation("service") ? "?" : "";
    const input = !operation.isUnary()
      ? operation.parameters.length == 0
        ? "void"
        : capitalize(iface.name) + capitalize(operation.name) + "Args"
      : expandType(operation.parameters[0].type, false);
    this.write(formatComment("  // ", operation.description));
    this.write(
      `  ${operation.name}${optionalSuffix}: Flow<${input}> | Step[],\n`,
    );
    super.triggerOperation(context);
  }

  visitInterfaceAfter(context: Context): void {
    this.write(`}\n\n`);
    super.triggerInterfaceAfter(context);
  }
}

class AuthsVisitor extends BaseVisitor {
  visitInterfaceBefore(context: Context): void {
    super.triggerInterfaceBefore(context);
    const { interface: iface } = context;
    this.write(formatComment("// ", iface.description));
    this.write(`export interface ${iface.name}Auth {\n`);
  }

  visitOperation(context: Context): void {
    const { operation } = context;
    this.write(`  ${camelCase(operation.name)}?: Authorization,\n`);
    super.triggerOperation(context);
  }

  visitInterfaceAfter(context: Context): void {
    this.write(`}\n\n`);
    super.triggerInterfaceAfter(context);
  }
}

class InterfaceVisitor extends BaseVisitor {
  visitInterfaceBefore(context: Context): void {
    super.triggerInterfaceBefore(context);
    const { namespace, interface: iface } = context;
    this.write(formatComment("// ", iface.description));
    this.write(`export const ${iface.name} = {
  $interface: "${namespace.name}.${iface.name}",\n`);
  }

  visitOperation(context: Context): void {
    const { namespace, interface: iface, operation } = context;
    this.write(formatComment("  // ", operation.description));
    this.write(
      `${operation.name}: "${namespace.name}.${iface.name}::${operation.name}" as Handler,\n`,
    );
    super.triggerOperation(context);
  }

  visitInterfaceAfter(context: Context): void {
    const { interface: iface } = context;
    const registerType = iface.annotation("service") ? "interface" : "provider";
    const as = iface.annotation("service") ? "as" : "as unknown as";
    this.write(`
  register(app: Application, iface: ${iface.name}Oper): void {
    app.${registerType}(
      ${iface.name}.$interface,
      iface ${as} Operations,
    );
  },

  authorize(app: Application, auths: ${iface.name}Auth): void {
    app.authorize(
      ${iface.name}.$interface,
      auths as Record<string, Authorization>
    );
  },
}\n\n`);
    super.triggerInterfaceAfter(context);
  }
}

class ClientVisitor extends BaseVisitor {
  visitInterfaceBefore(context: Context): void {
    super.triggerInterfaceBefore(context);
    const { interface: iface } = context;
    this.write(formatComment("// ", iface.description));
    this.write(`export const ${camelCase(iface.name)}Client = {\n`);
  }

  visitOperation(context: Context): void {
    const { interface: iface, operation } = context;
    const call = iface.annotation("service") ? "callInterface" : "callProvider";
    this.write(formatComment("  // ", operation.description));
    this.write(`${operation.name}(`);
    if (operation.isUnary()) {
      this.write(mapArg(operation.unaryOp()));
    } else {
      this.write(mapArgs(operation.parameters));
    }
    this.write(`): Response<${expandType(operation.type, true)}> {\n`);
    this.write("  const dataExpr = `{\n");
    operation.parameters.forEach(p => {
      this.write(` "${p.name}": \${toDataExpr(${p.name})}\n`)
    })
    this.write("}`;\n");
    this.write(
      `  return ${call}(${iface.name}.${operation.name}, dataExpr) as Response<${
        expandType(operation.type, true)
      }>;\n`,
    );
    this.write(`},\n\n`);
    super.triggerOperation(context);
  }

  visitInterfaceAfter(context: Context): void {
    this.write(`}\n\n`);
    super.triggerInterfaceAfter(context);
  }
}
