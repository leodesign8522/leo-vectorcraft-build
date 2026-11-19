//! Visible entry points and controls for the four path editing workflows.
use egui::Ui;
use serde_json::{Value,json};
use vectorcraft_tools::{Action,Mods,ToolKey};
use crate::VectorcraftApp;

pub const ID:&str="leoPathTools";
pub const TOOLS:[(&str,&str);4]=[
    ("directSelection","1 · PathScribe 手柄编辑"),
    ("extendPath","2 · Extend Path 路径延长"),
    ("anchorEraserBrush","3 · Smart Remove Brush 智能删点"),
    ("slideAnchor","4 · Reposition Point 节点滑移"),
];

#[derive(Clone)]
struct Transform {angle:f64,scale:f64}
impl Default for Transform {fn default()->Self{Self{angle:15.,scale:100.}}}

pub fn reveal_once(app:&mut VectorcraftApp) {
    if !app.ui.leo_path_tools_seen && app.ui.screen_mode<3 {
        app.ui.leo_path_tools_seen=true;
        app.prefs_changed=true;
        if let Err(e)=app.run("window.panel",json!({"panel":ID})) {app.ui.status=e;}
    }
}
fn run(app:&mut VectorcraftApp,command:&str,params:Value) {
    if let Err(e)=app.run(command,params) {app.ui.status=e;}
}
fn option(app:&mut VectorcraftApp,key:&str,value:Value) {
    run(app,"tool.setOption",json!({"key":key,"value":value}));
}
fn transform(app:&mut VectorcraftApp,operation:&str,angle:f64,scale:f64) {
    let o=app.session.tool_options();
    let result=app.run("path.leoTransformHandles",json!({
        "handles":o["leoHandles"],"revision":o["leoHandleRevision"],
        "operation":operation,"angle":angle,"scale":scale
    }));
    match result {
        Ok(_)=>{
            if let Err(e)=app.session.apply_actions(vec![Action::Notify("leo.handlesRefresh".into())]) {app.ui.status=e.to_string();}
        },
        Err(e)=>app.ui.status=e,
    }
}

pub fn show(app:&mut VectorcraftApp,ui:&mut Ui) {
    ui.heading("Leo Design Path Tools");
    ui.label("选择下面的工具，在画布上直接操作");
    let ready=app.session.active().is_some();
    ui.add_enabled_ui(ready,|ui| {
        for (id,label) in TOOLS {
            if ui.selectable_label(app.session.tool_id()==id,label).clicked() {
                let changed=app.session.tool_id()!=id;
                app.select_tool(id);
                if changed && id=="extendPath" {option(app,"mode",json!("bezier"));}
            }
        }
    });
    if !ready {ui.label("请先新建或打开文档。");return;}
    ui.separator();
    match app.session.tool_id() {
        "directSelection"=>{
            ui.label("点选手柄；Shift 增选；拖动已选手柄可一起移动。");
            if ui.button("框选手柄 (H)").clicked() {
                let view=app.view_info();
                let result=app.session.tool_key(ToolKey::Letter('h'),Mods::default(),view);
                crate::canvas::apply_requests(app,result);
            }
            let o=app.session.tool_options();
            let count=o["leoHandles"].as_array().map_or(0,Vec::len);
            let current=app.session.active().is_some_and(|d|Some(d.revision)==o["leoHandleRevision"].as_u64());
            ui.label(format!("已选手柄：{count}"));
            let can_edit=count>0 && current && !app.session.tool_busy();
            let key=egui::Id::new("leo-path-handle-transform");
            let mut settings=ui.ctx().data_mut(|d|d.get_temp::<Transform>(key).unwrap_or_default());
            ui.add_enabled_ui(can_edit,|ui| {
                ui.horizontal(|ui| {
                    if ui.button("批量等长").clicked() {transform(app,"equal",0.,1.);}
                    if ui.button("对称手柄").clicked() {transform(app,"symmetric",0.,1.);}
                });
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut settings.angle).range(-360.0..=360.0).suffix("°"));
                    if ui.button("旋转").clicked() {transform(app,"rotate",settings.angle,1.);}
                });
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut settings.scale).range(0.0..=1000.0).suffix("%"));
                    if ui.button("缩放").clicked() {transform(app,"scale",0.,settings.scale/100.);}
                });
            });
            ui.ctx().data_mut(|d|d.insert_temp(key,settings));
            if count>0 && !current {ui.label("文档已变化，请重新选择手柄。");}
        },
        "extendPath"=>{
            ui.label("拖动开放路径的任一端点。靠近其他端点时吸附。");
            let o=app.session.tool_options();
            ui.horizontal(|ui| {
                for (value,label) in [("bezier","贝塞尔"),("straight","直线"),("arc","圆弧")] {
                    if ui.selectable_label(o["mode"].as_str()==Some(value),label).clicked() {option(app,"mode",json!(value));}
                }
            });
            let mut locked=o["directionLocked"].as_bool().unwrap_or(false);
            if ui.checkbox(&mut locked,"沿端点切线锁定拖动方向 (Shift)").changed() {option(app,"directionLocked",json!(locked));}
        },
        "anchorEraserBrush"=>{
            ui.label("先选中路径，再刷过锚点；按容差重拟合，整笔可一次撤销。");
            let o=app.session.tool_options();
            let mut size=o["size"].as_f64().unwrap_or(20.);
            let mut tolerance=o["tolerance"].as_f64().unwrap_or(10.);
            ui.horizontal(|ui| {ui.label("直径");if ui.add(egui::DragValue::new(&mut size).range(1.0..=2500.0).suffix(" px")).changed() {option(app,"size",json!(size));}});
            ui.horizontal(|ui| {ui.label("容差");if ui.add(egui::DragValue::new(&mut tolerance).range(1.0..=100.0)).changed() {option(app,"tolerance",json!(tolerance));}});
            ui.label("容差 10 ≈ 0.2 pt；越小越保形。[ / ] 调整笔刷。");
        },
        "slideAnchor"=>{
            ui.label("拖动中间锚点沿相邻曲线滑移；开放路径端点保持不动。");
            let o=app.session.tool_options();
            let mut tolerance=o["tolerance"].as_f64().unwrap_or(0.2);
            ui.horizontal(|ui| {ui.label("拟合误差上限");if ui.add(egui::DragValue::new(&mut tolerance).range(0.001..=10.0).speed(0.01).suffix(" pt")).changed() {option(app,"tolerance",json!(tolerance));}});
            ui.label("超过误差上限时限制滑移距离；Shift 吸附切线角度。");
        },
        _=>{ui.label("点击上方按钮选择工具。");}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn texts(shape:&egui::Shape,out:&mut Vec<(String,egui::Pos2)>) {
        match shape {
            egui::Shape::Text(t)=>out.push((t.galley.text().to_string(),t.visual_bounding_rect().center())),
            egui::Shape::Vec(v)=>v.iter().for_each(|s|texts(s,out)),
            _=>{}
        }
    }
    fn frame(app:&mut VectorcraftApp,ctx:&egui::Context,events:Vec<egui::Event>)->Vec<(String,egui::Pos2)> {
        let mut output=ctx.run_ui(egui::RawInput{events,..Default::default()},|ui|show(app,ui));
        output.textures_delta.clear();
        let mut labels=vec![];
        for clipped in output.shapes {texts(&clipped.shape,&mut labels);}
        labels
    }
    #[test]
    fn leo_path_tools_panel_shows_and_selects_all_four_real_tools() {
        let mut app=VectorcraftApp::new(vectorcraft_engine::Session::new(),Default::default());
        app.run("file.new",json!({"width":400,"height":400})).unwrap();
        let ctx=egui::Context::default();
        for (id,label) in TOOLS {
            let labels=frame(&mut app,&ctx,vec![]);
            let pos=labels.iter().find(|(text,_)|text==label).unwrap_or_else(||panic!("missing {label}")).1;
            frame(&mut app,&ctx,vec![egui::Event::PointerMoved(pos),egui::Event::PointerButton{pos,button:egui::PointerButton::Primary,pressed:true,modifiers:egui::Modifiers::NONE}]);
            frame(&mut app,&ctx,vec![egui::Event::PointerButton{pos,button:egui::PointerButton::Primary,pressed:false,modifiers:egui::Modifiers::NONE}]);
            assert_eq!(app.session.tool_id(),id);
        }
        assert!(crate::state::ICON_PANELS.iter().any(|(id,_,_)|*id==ID));
        assert!(crate::state::ICON_PANEL_GROUPS.iter().any(|group|group.contains(&ID)));
        assert!(!app.ui.leo_path_tools_seen);
        reveal_once(&mut app);
        assert!(app.ui.leo_path_tools_seen);
    }
}
