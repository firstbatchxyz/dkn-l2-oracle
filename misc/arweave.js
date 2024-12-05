/**
 * A helper script to print the content of an Arweave transaction, where transaction id is hex-encoded.
 * This means that the input is a 64-char hexadecimal.
 *
 * Usage:
 *
 *  bun run ./misc/arweave.js 0x30613233613135613236663864663332366165306137663863633636343437336238373463353966333964623436366665316337313531393634623734393231
 *
 * Tip:
 *
 *  Can be piped to `pbcopy` on macOS to copy the output to clipboard.
 */

// parse input
let input = process.argv[2];
if (!input) {
  console.error("No input provided.");
}

// get rid of 0x
if (input.startsWith("0x")) {
  input = input.slice(2);
}

// decode to arweave url
const inputDecoded = Buffer.from(input, "hex").toString();
const arweaveTxid = Buffer.from(inputDecoded, "hex").toString("base64url");

// download the actual response from Arweave
const res = await fetch(`https://arweave.net/${arweaveTxid}`);
console.log(await res.text());
