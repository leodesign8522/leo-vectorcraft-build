"""Add the independent trim-and-join tool without changing original tool behavior."""
from pathlib import Path
import sys
root=Path(sys.argv[1]);source=Path(__file__).parent
def edit(path,old,new):
    p=root/path;s=p.read_text(encoding='utf-8')
    if s.count(old)!=1:raise SystemExit('Unexpected trim/join anchor: '+path+' '+old[:60])
    p.write_text(s.replace(old,new,1),encoding='utf-8',newline='\n')
(root/'crates/engine/src/cmd/path/leo_trim_join.rs').write_text((source/'trim_join.rs').read_text(encoding='utf-8'),encoding='utf-8',newline='\n')
(root/'crates/tools/src/addons/leo_trim_join.rs').write_text((source/'trim_join_tool.rs').read_text(encoding='utf-8'),encoding='utf-8',newline='\n')
edit('crates/engine/src/cmd/path.rs','mod leo_path_commands;','mod leo_path_commands;\nmod leo_trim_join;')
edit('crates/engine/src/cmd/path.rs','pub fn specs() -> Vec<CommandSpec> {\n    vec![','''pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!("path.leoTrimJoin", "Leo Design Trim and Join", [], None,
            "{points:[[x,y]...], join?:bool (true), preferSelected?:bool (true)} trim crossed intervals of editable expanded open paths at artwork intersections, optionally joining compatible unambiguous endpoints; at most 512 editable segments and 512 stroke points; one undo step",
            has_doc, leo_trim_join::run),''')
edit('crates/tools/src/addons.rs','pub mod anchor_eraser;','pub mod anchor_eraser;\npub mod leo_trim_join;')
edit('crates/tools/src/addons.rs','pub const PATH_TOOLS: &[ToolInfo] = &[','''pub const PATH_TOOLS: &[ToolInfo] = &[
    ToolInfo { id: leo_trim_join::ID, label: "Leo Trim and Join", shortcut: None, icon: "scissors" },''')
edit('crates/tools/src/addons.rs','        anchor_eraser::ID =>','        leo_trim_join::ID => Box::new(leo_trim_join::TrimJoinTool::default()),\n        anchor_eraser::ID =>')
with (root/'crates/engine/src/tests.rs').open('a',encoding='utf-8') as f:f.write((source/'trim_join_tests.rs').read_text(encoding='utf-8'))
