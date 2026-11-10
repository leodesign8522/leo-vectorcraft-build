// Exercise the production character mapper without requiring Illustrator on CI.
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict');
const source=fs.readFileSync(__dirname+'/illustrator_bridge.jsx','utf8');
const prefix=source.slice(0,source.indexOf('    var manifest=read('));
assert(prefix.includes('function character('));
const constants={DEFAULTFIGURESTYLE:0,TABULAR:1};
const ctx={app:{textFonts:[{family:'MiSans',style:'Bold',name:'MiSans-Bold'}]},FigureStyleType:constants,
  NoColor:function(){},RGBColor:function(){},FontCapsOption:{ALLCAPS:1},AutoKernType:{OPTICAL:1}};
vm.runInNewContext(prefix+'globalThis.mapCharacter=character;})();',ctx);
const attrs={},range={characterAttributes:attrs};
const style={font_family:'MiSans',font_style:'Bold',size:50.027,fill:{type:'none'},stroke:{type:'none'},features:['tnum']};
ctx.mapCharacter(range,style,{},1);assert.equal(attrs.figureStyle,constants.TABULAR);
assert.equal(attrs.textFont.name,'MiSans-Bold');
ctx.mapCharacter(range,{...style,features:[]},{},1);assert.equal(attrs.figureStyle,constants.DEFAULTFIGURESTYLE);
ctx.mapCharacter(range,{...style,features:undefined},{},1);assert.equal(attrs.figureStyle,constants.DEFAULTFIGURESTYLE);
assert.throws(()=>ctx.mapCharacter(range,{...style,features:['tnum','ss01']},{},1),/Unsupported OpenType feature: ss01/);
console.log('PASS: production tnum mapper, MiSans Bold, ordinary-run reset, absent features, unmapped feature rejection');
