const { transformSync } = require("@swc/core");
const { Visitor } = require("@swc/core/Visitor.js");
const fs = require('fs');
const process = require('process');

class WindowVisitor extends Visitor {
  visitMemberExpression(expression) {
    if (expression.object.type === 'MemberExpression') {
      this.visitMemberExpression(expression.object);
    }
    if (
      expression.object.type === 'Identifier'
      && expression.object.value === 'window'
      && expression.property.type === 'Identifier'
      && expression.property.value === 'location'
    ) {
      expression.property.value = 'reprise_location';
    }
    return expression;
  }
}

const args = process.argv.slice(2);
const filename = args[0];

// var start = new Date()
// var hrstart = process.hrtime()
fs.readFile(filename, 'utf8', (err, data) => {
  if (err) throw err;
  const transformed = transformSync(data, {
    plugin: (m) => new WindowVisitor().visitProgram(m),
  });
  fs.writeFile(`node-${filename}`, transformed.code, err => {
    if (err) throw err;
    // var end = new Date() - start,
    // hrend = process.hrtime(hrstart)
    console.log(`===${filename}===`)
    for (const [key,value] of Object.entries(process.memoryUsage())){
      console.log(`Memory usage by ${key}, ${value/1000000}MB`)
    }
    // console.info('Execution time: %dms', end)
    // console.info('Execution time (hr): %ds %dms\n', hrend[0], hrend[1] / 1000000)
  });
});