const { readFile, readdir, rm, writeFile } = require("fs/promises");

(async () => {
  const files = await readdir("./dist");
  const jsBundleName = files.find((fileName) => fileName.endsWith("js"));

  const jsBundleBuffer = await readFile(`./dist/${jsBundleName}`);
  let jsBundleString = jsBundleBuffer.toString();

  jsBundleString = jsBundleString.slice(jsBundleString.indexOf(";") + 1);

  const { 1: toReplace, index } = jsBundleString.match(
    /imports\['env'\] = (.*?);/
  );

  const res = `${jsBundleString.slice(0, index + "imports['env'] = ".length)}{
    rustsecp256k1_v0_4_1_context_preallocated_size: () => null,
    rustsecp256k1_v0_4_1_context_preallocated_create: () => null,
    rustsecp256k1_v0_4_1_context_preallocated_destroy: () => null,
  }${jsBundleString.slice(
    index + "imports['env'] = ".length + toReplace.length
  )}`;

  await rm(`./dist/${jsBundleName}`);
  await writeFile(`./dist/${jsBundleName}`, res);
})();
