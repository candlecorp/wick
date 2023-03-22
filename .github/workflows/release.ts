function env(name: string): string {
    const val = Deno.env.get(name);
    if (!val) throw new Error(`$${name} is required`);
    return val;
}

console.log(env("FILES"));

const files = env("FILES").split(/\s*,\s*/);
const version = env("VERSION");

if (!version.match(/^\d+\.\d+\.\d+$/)) {
    throw new Error("VERSION must be in the form 1.2.3");
}

console.log("Updating versions in files");

for (const file of files) {
    console.log(`Updating ${file} for ${version}...`);
    const orig = Deno.readTextFileSync(file);
    const updated = orig.replace(/(version\s*=\s*")\d+\.\d+\.\d+(")/, `$1${version}$2`);
    Deno.writeFileSync(file, new TextEncoder().encode(updated));
}