// A build plugin that reads and parses the publication view and contributes nothing.
// parity:build-bench measures the process protocol's cost with it.
let input = '';
process.stdin.setEncoding('utf8');
for await (const chunk of process.stdin) input += chunk;
const request = JSON.parse(input);
if (request.apiVersion !== 1) throw new Error(`unexpected apiVersion ${request.apiVersion}`);
process.stdout.write(JSON.stringify({ apiVersion: 1, contributions: [] }));
