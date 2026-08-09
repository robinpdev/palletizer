import { readFileSync, writeFileSync, cpSync } from "fs";

const dirName = "./rust/pkg/"; // change this to match your Rust library's name

const content = readFileSync(dirName + "package.json");

const packageJSON = JSON.parse(String(content));
packageJSON["type"] = "module";

writeFileSync(dirName + "package.json", JSON.stringify(packageJSON));

try {
	cpSync("./rust/pkg", "./node_modules/rust", { recursive: true });
} catch (e) {
	console.error("Failed to copy to node_modules/rust", e);
}
