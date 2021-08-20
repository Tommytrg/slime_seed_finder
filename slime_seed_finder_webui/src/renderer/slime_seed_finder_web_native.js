"use strict";

if (typeof Rust === "undefined") {
    var Rust = {};
}

Rust.slime_seed_finder_web = new Promise((resolve, reject) => {
    setTimeout(() => {
        let rust_addon = require("../../rust-dist");
        console.log(
            "Loaded rust addon in slime_seed_finder_web_native.js:",
            rust_addon
        );
        rust_addon.init(function(args) {
            let level = args.shift();
            console.log(...args);
        });
        resolve(rust_addon);
    }, 0);
});
//Rust.slime_seed_finder_web = Promise.resolve(
//    rust_addon
//)
