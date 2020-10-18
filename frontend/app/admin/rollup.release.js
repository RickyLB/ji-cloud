import rust from "@wasm-tool/rollup-plugin-rust";
const {URL_FRONTEND_RELEASE} = require("../../../config/js");

const NAME = "admin";

export default {
    input: {
        index: "./Cargo.toml",
    },
    output: {
        entryFileNames: "[name]-[hash].js",
        dir: "dist/js/",
        format: "iife",
        sourcemap: true,
    },
    plugins: [
        rust({
            serverPath: `${URL_FRONTEND_RELEASE}/${NAME}/js/`,
			cargoArgs: ["--features", "release"],
            debug: false,
        }),
    ],
};
