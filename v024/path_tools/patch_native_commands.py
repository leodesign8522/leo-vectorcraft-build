"""Register conservative engine commands; does not claim brush/drag UI support."""
from pathlib import Path
import sys

root=Path(sys.argv[1])
source=Path(__file__).parent
dest=root/'crates/engine/src/cmd/path'
dest.mkdir(exist_ok=True)
(dest/'leo_path_commands.rs').write_text((source/'native_commands.rs').read_text(encoding='utf-8'),encoding='utf-8',newline='\n')
(dest/'leo_geometry.rs').write_text((source/'geometry.rs').read_text(encoding='utf-8'),encoding='utf-8',newline='\n')
p=root/'crates/engine/src/cmd/path.rs'
s=p.read_text(encoding='utf-8')
anchor='pub fn specs() -> Vec<CommandSpec> {'
if s.count(anchor)!=1: raise SystemExit('Unexpected path command specs')
s=s.replace(anchor,'mod leo_path_commands;\n\n'+anchor,1)
anchor='    vec![\n'
if s.count(anchor)<1: raise SystemExit('Missing path specs list')
specs=''
for command,label,params,run in (
    ('leoTransformHandles','Transform Handles','{handles:[{id,subpath?,anchor,which,x?,y?}], operation?:rotate|scale|equal|symmetric, angle?:degrees, scale?:number, revision?:number}','transform_handles'),
    ('leoExtend','Extend Path','{id, subpath?:usize, mode?:tangent|cubic, length?:number, amount?:number}','extend'),
    ('leoSmartRemove','Remove Join Within Tolerance','{id, subpath?:usize, anchor:usize, tolerance?:number}','remove'),
    ('leoReposition','Reposition Join','{id, subpath?:usize, anchor:usize, t?:number, tolerance?:number}','reposition'),
):
    specs+=f'        cmd!("path.{command}", "Leo Design {label}", [], None, "{params}", has_doc, leo_path_commands::{run}),\n'
s=s.replace(anchor,anchor+specs,1)
p.write_text(s,encoding='utf-8',newline='\n')
with (root/'crates/engine/src/tests.rs').open('a',encoding='utf-8') as f:
    f.write((source/'native_commands_tests.rs').read_text(encoding='utf-8'))
    f.write((source/'four_tools_engine_tests.rs').read_text(encoding='utf-8'))
