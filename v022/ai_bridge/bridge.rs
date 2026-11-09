//! Local, opt-in Illustrator selection inbox. No socket or global hotkey service.
use std::path::{Path,PathBuf};
use std::time::{Duration,Instant,SystemTime,UNIX_EPOCH};
use serde_json::{Value,json};
use crate::VectorcraftApp;

const SENDER:&str=include_str!("Send_to_VectorCraft.jsx");
pub fn root()->Result<PathBuf,String> {
    std::env::var_os("LOCALAPPDATA").map(|p|PathBuf::from(p).join("Leo Design").join("AI Bridge"))
        .ok_or("找不到 Windows 本地应用数据目录。".into())
}
fn id()->String {
    format!("{}_{}",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos())
}
pub(super) fn outbound()->Result<PathBuf,String> {
    let folder=root()?.join("outbox").join(id());
    vectorcraft_engine::file_access::check_write(&folder.to_string_lossy())?;
    std::fs::create_dir_all(&folder).map_err(|e|e.to_string())?;
    Ok(folder.join("VectorCraft_selection.ai"))
}
pub fn send(app:&mut VectorcraftApp,p:&Value)->Result<Value,String> {
    let result=(||{
        let st=app.session.active().ok_or("请先打开文档并选中对象。")?;
        if st.selection.objects.is_empty() {return Err("请先用 V 工具选中要发送的完整对象或编组。".into());}
        if !st.selection.anchors.is_empty() || !st.selection.segments.is_empty() {
            return Err("第一版发送完整对象或编组。请按 V 重新选择，避免把局部节点误当作整个对象发送。".into());
        }
        if app.background.busy_with(st.uid) {return Err("当前文档正在保存或发送，请等待完成后重试。".into());}
        let mut options=json!({"selectedOnly":true,"handoff":true});
        options["wait"]=p.get("wait").cloned().unwrap_or(json!(false));
        // Allocate output only after all selection/preflight checks in the exporter.
        super::ai_export::export(app,&options)
    })();
    if let Err(e)=&result {crate::dialogs::confirm::tell(app,"发送到 Illustrator 失败",e);}
    result
}

/// A malformed or external inbox entry must never turn into an arbitrary file open.
fn input_file(job:&Path,inbox:&Path)->Result<PathBuf,String> {
    let name=job.file_name().and_then(|n|n.to_str()).ok_or("无效的接收任务名称")?;
    if name.is_empty() || !name.bytes().all(|c|c.is_ascii_digit()||c==b'_') {return Err("无效的接收任务名称".into());}
    let base=std::fs::canonicalize(inbox).map_err(|e|e.to_string())?;
    let canonical=std::fs::canonicalize(job).map_err(|e|e.to_string())?;
    if canonical.parent()!=Some(base.as_path()) {return Err("接收任务超出允许目录".into());}
    let input=std::fs::canonicalize(canonical.join("selection.ai")).map_err(|e|e.to_string())?;
    if input.parent()!=Some(canonical.as_path()) {return Err("接收文件超出任务目录".into());}
    vectorcraft_engine::file_access::check_read(&input.to_string_lossy())?;
    let size=std::fs::metadata(&input).map_err(|e|e.to_string())?.len();
    if size>256*1024*1024 {return Err("选中内容超过 256MB，请分批发送。".into());}
    Ok(input)
}
fn receive(app:&mut VectorcraftApp,input:&Path)->Result<Value,String> {
    let bytes=std::fs::read(input).map_err(|e|e.to_string())?;
    // A new untitled document protects the Illustrator source and exchange file.
    let mut result=crate::io::open_document(app,"Illustrator selection.ai",&bytes,None,
        &json!({"editingData":"always","layers":true,"textAs":"text"}))?;
    let notes=result["warnings"].as_array().cloned().unwrap_or_default();
    let native=result["editingDataRead"]==true;
    let mut text=if native {"已读取 Illustrator 编辑数据，接收为新文档。".to_string()}
        else {"未读取到完整 Illustrator 编辑数据；请核对文字和编组。".to_string()};
    text.push_str("\nRGB/CMYK 色值按交换文件读取，请核对两端工作配置文件。");
    for note in notes.iter().filter_map(Value::as_str) {text.push('\n');text.push_str(note);}
    // First receive is explicit about fidelity; subsequent warning-free transfers stay quiet.
    if !native || !notes.is_empty() {
        crate::dialogs::confirm::tell(app,"Illustrator 接收提示",&text);
    }
    app.status(if notes.is_empty() {"已接收 Illustrator 选中对象。"}else{"已接收，请查看兼容提示。"});
    result["bridgeDirection"]=json!("illustrator-to-vectorcraft");
    Ok(result)
}
pub fn poll(app:&mut VectorcraftApp,ctx:&egui::Context) {
    if !cfg!(windows) || app.services.read.is_none() {return;}
    let key=egui::Id::new("leo-ai-bridge-inbox-poll");
    let now=Instant::now();
    if ctx.data(|d|d.get_temp::<Instant>(key)).is_some_and(|last|now.duration_since(last)<Duration::from_millis(750)) {
        ctx.request_repaint_after(Duration::from_millis(750));return;
    }
    ctx.data_mut(|d|d.insert_temp(key,now));
    let Ok(root)=root() else{return};
    let config=root.join("receiver.txt");
    let Ok(receiver)=std::fs::read_to_string(&config) else{return};
    let Ok(exe)=std::env::current_exe() else{return};
    if !receiver.trim_start_matches('\u{feff}').trim().eq_ignore_ascii_case(&exe.to_string_lossy()) {return;}
    if vectorcraft_engine::file_access::check_write(&root.to_string_lossy()).is_err() {return;}
    let stamp=SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis().to_string();
    let _=std::fs::write(root.join("heartbeat.txt"),stamp);
    ctx.request_repaint_after(Duration::from_millis(750));
    if app.ui.dialog.is_some() || app.file_dialog_open() || !app.background.jobs.is_empty() {return;}
    let inbox=root.join("inbox");
    let Ok(entries)=std::fs::read_dir(&inbox) else{return};
    let mut jobs:Vec<_>=entries.flatten().filter(|e|e.file_type().is_ok_and(|t|t.is_dir())).map(|e|e.path()).collect();
    jobs.sort();
    for job in jobs {
        let ready=job.join("ready.json");
        if !ready.is_file() {continue;}
        let owned=std::fs::canonicalize(&job).ok();
        let parent=std::fs::canonicalize(&inbox).ok();
        if owned.as_ref().and_then(|j|j.parent())!=parent.as_deref() {continue;}
        // Atomic claim means two application windows cannot receive the same job.
        let claim=job.join("receiving.json");
        if std::fs::rename(&ready,&claim).is_err() {continue;}
        let result=input_file(&job,&inbox).and_then(|path|receive(app,&path));
        let report=match result {
            Ok(v)=>{ctx.send_viewport_cmd(egui::ViewportCommand::Focus);json!({"pass":true,"result":v})}
            Err(error)=>{crate::dialogs::confirm::tell(app,"Illustrator 接收失败",&error);json!({"pass":false,"error":error})}
        };
        let _=std::fs::write(job.join("receipt.json"),serde_json::to_vec(&report).unwrap_or_default());
        let _=std::fs::rename(&claim,job.join("processed.json"));
        break;
    }
}
pub fn sender_source()->&'static str {SENDER}

#[cfg(test)]
mod leo_ai_bridge_tests {
    use super::*;
    use vectorcraft_engine::Session;
    #[test]
    fn no_selection_is_reported_without_launching_illustrator(){
        let mut app=VectorcraftApp::new(Session::new(),crate::Services::default());
        assert!(send(&mut app,&json!({})).is_err());
        assert!(app.ui.dialog.is_some());
        app.session.execute("file.new",&json!({})).unwrap();
        assert!(send(&mut app,&json!({})).is_err());
        assert_eq!(app.session.documents().len(),1);
    }
    #[test]
    fn inbox_refuses_external_job_and_symlink_target(){
        let base=std::env::temp_dir().join(format!("leo-bridge-test-{}",id()));
        let inbox=base.join("inbox");let good=inbox.join("123_456");let bad=base.join("789");
        std::fs::create_dir_all(&good).unwrap();std::fs::create_dir_all(&bad).unwrap();
        std::fs::write(good.join("selection.ai"),b"%PDF-test").unwrap();
        std::fs::write(bad.join("selection.ai"),b"%PDF-test").unwrap();
        assert!(input_file(&good,&inbox).is_ok());
        assert!(input_file(&bad,&inbox).is_err());
        assert!(input_file(&inbox.join("../789"),&inbox).is_err());
        for file in [good.join("selection.ai"),bad.join("selection.ai")] {std::fs::remove_file(file).unwrap();}
        for dir in [&good,&bad,&inbox,&base] {std::fs::remove_dir(dir).unwrap();}
    }
}
