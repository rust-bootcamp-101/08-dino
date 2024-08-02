import { execute } from "./lib.ts";

export default async function main() {
    console.log(`Executing main`);
    console.log(await execute('world'));
}
