const assert = require('node:assert/strict');
const fs = require('node:fs');
const vm = require('node:vm');
const source = fs.readFileSync(__dirname + '/illustrator_bridge.jsx', 'utf8');
const start = source.indexOf('        function capture(from)');
const end = source.indexOf('        var layers=[];',start);
assert(start >= 0 && end > start);
const capture = vm.runInNewContext(source.slice(start,end) + '\ncapture;');
const flat = {name:'flat',layers:[],pageItems:[]};
function item(parent,name,z){
    return {name,parent,geometricBounds:[0,0,1,-1],locked:false,hidden:false,
        get zOrderPosition(){ if(z === undefined) throw Error('Internal error'); return z; }};
}
flat.pageItems = [item(flat,'front'), item({},'nested'), item(flat,'back')];
assert.equal(Array.from(capture(flat).entries,e=>e.item.name).reverse().join(','),'front,back');
const nested = {name:'mixed', layers:[{name:'middle',layers:[],pageItems:[],zOrderPosition:1}],pageItems:[]};
nested.pageItems = [item(nested,'front',0),item(nested,'back',2)];
assert.equal(Array.from(capture(nested).entries,e=>e.layer?e.data.name:e.item.name).reverse().join(','),'front,middle,back');
console.log('PASS: flat layer paths with unavailable stacking positions; mixed sibling order');
