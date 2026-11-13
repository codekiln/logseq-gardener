import { readFileSync, writeFileSync } from 'node:fs';

export const runnerRepair = 'Pass each note format to mldoc reference extraction: extractRefs(ast, format).';

export function repairRunner(file) {
  const source = readFileSync(file, 'utf8');
  const original = 'refs: extractRefs(ast)';
  if (source.split(original).length !== 2) {
    throw Error('Expected exactly one unpatched mldoc reference call in the pinned lsdoc runner. Review the source revision and repair.');
  }
  writeFileSync(file, source.replace(original, 'refs: extractRefs(ast, format)'));
}
