const {
  NodeModulesPolyfillPlugin,
} = require("@esbuild-plugins/node-modules-polyfill");
const { build } = require("esbuild");

build({
  entryPoints: ["ts/app.ts"],
  outfile: "bundle.js",
  platform: "browser",
  format: "cjs",
  minify: true,
  bundle: true,
  treeShaking: true,
  plugins: [NodeModulesPolyfillPlugin()],
})
  .then(console.log)
  .catch(() => process.exit(1));
