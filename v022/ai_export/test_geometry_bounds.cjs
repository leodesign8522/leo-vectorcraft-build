// Test the production bounds validator with source coordinates matching the
// screenshot. This runs without Illustrator; real DOM round-trip remains separate.
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(__dirname+'/illustrator_bridge.jsx','utf8');
const start=source.indexOf('// LEO_GEOMETRY_VALIDATION_BEGIN');
const end=source.indexOf('// LEO_GEOMETRY_VALIDATION_END');
assert(start>=0&&end>start,'production geometry validation missing');
const context={};vm.runInNewContext(source.slice(start,end),context);
function doc(board,items,scale=1){return {scaleFactor:scale,artboards:[{artboardRect:board}],pageItems:items.map(b=>({geometricBounds:b}))};}
const witness={bounds:[14034,-422,14546,-192],artboard:[14033,-423,14547,-191]};
for(const scale of [1,10]){
  const board=[1000,2000,1000+514/scale,2000-232/scale];
  const expected=context.expectedGeometryBounds(board,witness,scale);
  const good=doc(board,[expected],scale);
  assert.equal(context.validateGeometryBounds(good,witness).pass,true);
  const before=JSON.stringify(good);
  context.validateGeometryBounds(good,witness);
  assert.equal(JSON.stringify(good),before,'validator must not alter artwork');
  // A phantom point/rectangle to the left produces the wide selection in the report.
  const phantom=[expected[0]-1300/scale,expected[1],expected[0]-1300/scale,expected[3]];
  assert.throws(()=>context.validateGeometryBounds(doc(board,[expected,phantom],scale),witness),/选框范围校验失败/);
  assert.throws(()=>context.validateGeometryBounds(doc(board,[],scale),witness),/未生成/);
  assert.throws(()=>context.validateGeometryBounds(doc(board,[[NaN,1,2,3]],scale),witness),/范围无效/);
}
assert.equal(context.validateGeometryBounds(null,null),null,'complex artwork is outside the simple-path witness');
console.log('PASS: far/negative coordinates, scale 1/10, phantom bounds rejection, no artwork mutation');
