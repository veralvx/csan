// Node demo for csan-web.
//
// Build the Node package first, then run this file:
//
//     wasm-pack build --release --target nodejs --out-dir pkg-node
//     node examples/node-demo.cjs

const {
  cutterSanborn,
  cutterSanbornDetailed,
  cutterLoc,
} = require("../pkg-node/csan_web.js");

const works = [
  { givenName: "Charles", familyName: "Dickens", title: "Oliver Twist" },
  { familyName: "Borges", title: "Ficciones" },
  { title: "100 best hamburger recipes" },
];

for (const work of works) {
  const label = work.familyName ?? work.title;
  console.log(
    `${label}: sanborn=${cutterSanborn(work)} loc=${cutterLoc(work)}`,
  );
}

console.log("detailed:", cutterSanbornDetailed({ familyName: "Borges" }));
