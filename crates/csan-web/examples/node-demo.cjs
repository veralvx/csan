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
  { forename: "Charles", surname: "Dickens", title: "Oliver Twist" },
  { surname: "Borges", title: "Ficciones" },
  { title: "100 best hamburger recipes" },
];

for (const work of works) {
  const label = work.surname ?? work.title;
  console.log(
    `${label}: sanborn=${cutterSanborn(work)} loc=${cutterLoc(work)}`,
  );
}

console.log("detailed:", cutterSanbornDetailed({ surname: "Borges" }));
