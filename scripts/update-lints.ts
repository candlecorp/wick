import fs from "fs";
import path from "path";

const crateDir = path.join(__dirname, "..", "crates");

const crates = fs.readdirSync(crateDir);

const lintsPath = path.join(__dirname, "..", "etc", "lints.rs");

const replaceRegex = /(!!START_LINTS.*!!END_LINTS)/s;

const completeLints = fs.readFileSync(lintsPath, "utf-8");
const match = completeLints.match(replaceRegex);
if (!match) {
  throw new Error(`Could not parse out replacable portion of ${lintsPath}`);
}
const denyOnlyLints = match[0];

const replacementCandidates = ["lib.rs", "main.rs"];

crates
  .filter((crate) => !crate.startsWith("test"))
  .forEach((crate) => {
    replacementCandidates.forEach((file) => {
      const relativePath = path.join(crate, "src", file);
      const candidatePath = path.join(crateDir, relativePath);
      if (!fs.existsSync(candidatePath)) {
        // console.log(`No file found at ${candidatePath}...`);
      } else {
        let rustSrc = fs.readFileSync(candidatePath, "utf-8");
        let newSrc = rustSrc;
        if (rustSrc.match(replaceRegex)) {
          console.log(`UPDATING: ${relativePath}`);
          newSrc = rustSrc.replace(replaceRegex, denyOnlyLints);
        } else {
          console.log(`ADDING: ${relativePath}`);
          newSrc = [completeLints, rustSrc].join("\n");
        }
        fs.writeFileSync(candidatePath, newSrc);
      }
    });
  });
