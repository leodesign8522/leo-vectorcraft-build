const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');

const source = fs.readFileSync(__dirname + '/illustrator_bridge.jsx', 'utf8');
const start = source.indexOf('    function paragraphJustification(');
const end = source.indexOf('    function path(', start);
assert(start >= 0 && end > start);
const Justification = {
    LEFT: 'left',
    CENTER: 'center',
    RIGHT: 'right',
    FULLJUSTIFYLASTLINELEFT: 'justify-left',
    FULLJUSTIFYLASTLINECENTER: 'justify-center',
    FULLJUSTIFYLASTLINERIGHT: 'justify-right',
    FULLJUSTIFY: 'justify-all',
};
const paragraph = vm.runInNewContext(source.slice(start, end) + '\nparagraph;', { Justification });
for (const [input, point, area] of [
    ['Auto', 'left', 'left'],
    ['Left', 'left', 'left'],
    ['Center', 'center', 'center'],
    ['Right', 'right', 'right'],
    ['JustifyLeft', 'left', 'justify-left'],
    ['JustifyCenter', 'center', 'justify-center'],
    ['JustifyRight', 'right', 'justify-right'],
    ['JustifyAll', 'left', 'justify-all'],
]) {
    for (const [kind, expected] of [['point', point], ['area', area]]) {
        const attrs = {};
        paragraph(attrs, { justify: input }, 1, kind);
        assert.equal(attrs.justification, expected, kind + ' ' + input);
    }
}
console.log('PASS: point text alignment follows VectorCraft anchors; area text retains full justification');
