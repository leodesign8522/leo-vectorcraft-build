//! Editable AI handoff: native geometry/colour/layers plus live point/area type.
//! Illustrator writes the final AI. User documents are never edited by this job.
use serde_json::{Value, json};
use vectorcraft_color::{Color, Paint, Swatch};
use vectorcraft_doc::{Appearance, CharStyle, Document, Node, NodeKind, TextKind};
use vectorcraft_geom::{Point, Rect, shapes};
use crate::{FilePick, VectorcraftApp};

const LABEL: &str = "导出可编辑 AI（通过 Illustrator）";
const SCRIPT: &str = include_str!("illustrator_bridge.jsx");

struct Prepared { doc: Document, texts: Vec<Value> }

fn simple_paint(p: &Paint) -> bool { matches!(p, Paint::None | Paint::Solid { .. }) }

fn check_text(n: &Node) -> Result<(), String> {
    let NodeKind::Text(t) = &n.kind else { return Ok(()) };
    let fail = |why: &str| format!("文字对象 {}：{why}。请先简化该对象，或使用普通 AI 导出并接受转轮廓。", n.id.0);
    if !matches!(t.kind, TextKind::Point | TextKind::Area { .. }) { return Err(fail("暂不支持将路径文字写为活文字")); }
    if !t.ruby.is_empty() || !t.wrap.is_empty() { return Err(fail("暂不支持注音或对象绕排")); }
    if n.blend!=Default::default() { return Err(fail("暂不支持文字自身的特殊混合模式")); }
    if !n.appearance.items.is_empty() || !n.appearance.effects.is_empty() || n.extras().mask.is_some() { return Err(fail("文字带有额外外观或蒙版")); }
    if let TextKind::Area { frame } = &t.kind {
        if frame.subpaths.len()!=1 || !frame.subpaths[0].closed { return Err(fail("文字框需要单一闭合路径")); }
        let area=&t.area;
        if !matches!(area.first_baseline,vectorcraft_doc::FirstBaseline::Ascent)
            || area.first_baseline_min!=0.0 || serde_json::to_value(area.vertical_align).ok()!=Some(json!("top")) {
            return Err(fail("暂不支持自定义首基线或垂直对齐模式"));
        }
    }
    let x=t.xf.as_coeffs();
    if !x.iter().all(|v|v.is_finite()) || (x[0]*x[3]-x[1]*x[2]).abs()<1e-12 { return Err(fail("文字变换不可逆")); }
    for run in &t.runs {
        if run.inline.is_some() { return Err(fail("暂不支持行内图形")); }
        let s=&run.style;
        if !simple_paint(&s.fill) || !simple_paint(&s.stroke) { return Err(fail("活文字只支持普通填色和专色")); }
        // Check every other character field against its default rather than silently
        // discarding advanced shaping, variable axes or custom glyph substitutions.
        let mut supported=CharStyle::default();
        supported.font_family=s.font_family.clone();supported.font_style=s.font_style.clone();
        supported.size=s.size;supported.leading=s.leading;supported.tracking=s.tracking;
        supported.kerning=s.kerning;supported.optical_kerning=s.optical_kerning;
        supported.baseline_shift=s.baseline_shift;supported.h_scale=s.h_scale;supported.v_scale=s.v_scale;
        supported.rotation=s.rotation;supported.fill=s.fill.clone();supported.stroke=s.stroke.clone();
        supported.stroke_width=s.stroke_width;supported.underline=s.underline;supported.strikethrough=s.strikethrough;
        supported.all_caps=s.all_caps;supported.no_break=s.no_break;
        supported.overprint_fill=s.overprint_fill;supported.overprint_stroke=s.overprint_stroke;
        supported.style_name=s.style_name.clone();
        if &supported!=s { return Err(fail("存在尚未支持的高级字符属性（如可变字体轴、注音字形或特殊字符位置）")); }
    }
    for para in std::iter::once(&t.para).chain(t.paras.iter()) {
        if para.list.is_some() || !para.tabs.is_empty() || para.direction.is_some() {
            return Err(fail("暂不支持列表、制表位或显式双向段落"));
        }
    }
    Ok(())
}

fn prepare(doc: &Document, token: &str) -> Result<Prepared, String> {
    let assigned_profile=match doc.color_mode {vectorcraft_doc::ColorMode::Cmyk=>&doc.color_profiles.cmyk,_=>&doc.color_profiles.rgb};
    if assigned_profile.is_some() { return Err("文档指定了 ICC 配置文件；当前可编辑 AI 导出支持 RGB/CMYK 模式和数值，但尚不能保证指定 ICC 的继承。请保留原文件，先在副本中改用工作空间后再导出。".into()); }
    if !doc.text_threads.is_empty() { return Err("串接文字框暂不支持活文字 AI 导出，请先在副本中拆分串接。".into()); }
    let mut texts=vec![];let mut errors=vec![];
    doc.walk(|n| if matches!(n.kind, NodeKind::Text(_)) {
        if let Err(e)=check_text(n) { errors.push(e); }
        for parent in doc.ancestry(n.id).unwrap_or_default().into_iter().filter(|id|*id!=n.id).filter_map(|id|doc.node(id)) {
            if parent.extras().mask.is_some() || !parent.appearance.items.is_empty() || !parent.appearance.effects.is_empty() {
                errors.push(format!("文字对象 {} 的父容器有额外外观或透明度蒙版，请先在副本中简化。",n.id.0));
            }
            if let NodeKind::Group{children,clip:true}|NodeKind::Layer{children,clip:true,..}=&parent.kind {
                if children.first().is_some_and(|c|c.id==n.id) {errors.push("作为剪切路径的文字暂不支持活文字 AI 导出。".into());}
            }
        }
        let marker=format!("LEO_AI_TEXT_{token}_{}",n.id.0);
        texts.push(json!({"marker":marker,"node":n}));
    });
    if !errors.is_empty() { return Err(errors.into_iter().take(8).collect::<Vec<_>>().join("\n")); }
    // Text inside symbols/patterns cannot be mapped one-to-one to story objects.
    for symbol in doc.symbols.iter() {
        let mut has_text=false;symbol.art.walk(&mut |n|has_text|=matches!(n.kind,NodeKind::Text(_)));
        if has_text { return Err("符号中含有文字，请先在副本中展开符号后导出可编辑 AI。".into()); }
    }
    for pattern in doc.patterns.iter() {
        let mut has_text=false;for art in &pattern.art {art.walk(&mut |n|has_text|=matches!(n.kind,NodeKind::Text(_)));}
        if has_text {return Err("图案色板中含有文字，请先在副本中展开图案后导出可编辑 AI。".into());}
    }
    let mut marked=doc.without_edit_modes().into_owned();
    for entry in &texts {
        let id=vectorcraft_doc::NodeId(entry["node"]["id"].as_u64().ok_or("文字 ID 无效")?);
        let original=doc.node(id).ok_or("文字对象缺失")?;
        let NodeKind::Text(t)=&original.kind else { return Err("文字类型变化".into()) };
        let origin=t.xf*Point::ZERO;
        let name=entry["marker"].as_str().ok_or("文字标记缺失")?.to_owned();
        let c=Color::rgb(1.0,0.0,1.0);
        marked.swatches.push(Swatch {name:name.clone(),paint:Paint::solid(c.clone()),global:true,spot:true});
        let paint=Paint::Solid {color:c,swatch:Some(name),tint:1.0};
        let mut marker=Node::path(id,shapes::rectangle(Rect::new(origin.x,origin.y,origin.x+1.0,origin.y+1.0)),Appearance::basic(paint,Paint::None,0.0));
        marker.visible=original.visible;
        *marked.node_mut(id).ok_or("文字位置缺失")?=marker;
    }
    Ok(Prepared {doc:marked,texts})
}

pub fn export(app: &mut VectorcraftApp,p:&Value)->Result<Value,String>{
    if p.get("wait").and_then(Value::as_bool)==Some(true) {
        return crate::background::in_foreground(app,|app|export_now(app,p));
    }
    export_now(app,p)
}

fn export_now(app:&mut VectorcraftApp,p:&Value)->Result<Value,String>{
    if !cfg!(windows) { return Err("可编辑 AI 导出需要 Windows 和已安装的 Illustrator。".into()); }
    let st=app.session.active().ok_or("请先打开文档。")?;
    let uid=st.uid;let doc=st.doc.clone();
    // Report unsupported content before opening a save picker or launching Illustrator.
    let token=format!("{}_{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos());
    let prepared=prepare(&doc,&token)?;
    let path=p.get("path").and_then(Value::as_str).map(str::to_owned).or_else(||crate::picks::save(app,&FilePick {
        filters:vec![FilePick::filter("Editable Illustrator AI",&["ai"])],..FilePick::named("可编辑文字.ai")
    }));
    let Some(path)=path else {return Ok(json!({"cancelled":true}));};
    if !path.to_ascii_lowercase().ends_with(".ai") { return Err("请选择 .ai 文件名。".into()); }
    vectorcraft_engine::file_access::check_write(&path)?;
    let output=std::path::PathBuf::from(&path);
    let parent=output.parent().filter(|p|!p.as_os_str().is_empty()).unwrap_or(std::path::Path::new("."));
    let temp=parent.join(format!(".leo-ai-export-{token}"));
    vectorcraft_engine::file_access::check_write(&temp.to_string_lossy())?;
    let mode=match doc.color_mode {vectorcraft_doc::ColorMode::Cmyk=>"cmyk",_=>"rgb"};
    crate::background::run(app,true,LABEL.into(),Some(uid),move |write|{
        std::fs::create_dir(&temp).map_err(|e|format!("无法建立导出临时目录：{e}"))?;
        let result=(||{
            let (bytes,warnings)=vectorcraft_engine::cmd::fileio::encode_with_warnings(&prepared.doc,"ai",&json!({"format":"ai","editingData":true,"pdfCompatible":true})).map_err(|e|e.to_string())?;
            let input=temp.join("structure.ai");let final_ai=temp.join("editable.ai");let manifest=temp.join("manifest.json");let report=temp.join("report.json");let script=temp.join("bridge.jsx");
            write(&input.to_string_lossy(),&bytes)?;write(&script.to_string_lossy(),SCRIPT.as_bytes())?;
            let data=json!({"version":1,"mode":mode,"inputAi":input,"outputAi":final_ai,"reportPath":report,"texts":prepared.texts});
            write(&manifest.to_string_lossy(),&serde_json::to_vec(&data).map_err(|e|e.to_string())?)?;
            run_illustrator(&script,&manifest)?;
            let proof:Value=serde_json::from_slice(&std::fs::read(&report).map_err(|e|format!("Illustrator 未返回验证报告：{e}"))?).map_err(|e|e.to_string())?;
            if proof["pass"]!=true || proof["mode"]!=mode {return Err(format!("AI 导出验证失败：{}",proof["error"]));}
            let bytes=std::fs::read(&final_ai).map_err(|e|e.to_string())?;
            if !bytes.starts_with(b"%PDF-") {return Err("Illustrator 未生成 PDF 兼容 AI 文件。".into());}
            // Only the successfully generated, verified result reaches the user's path.
            write(&path,&bytes)?;
            Ok(json!({"path":path,"mode":mode,"bytes":bytes.len(),"textFrames":proof["texts"].as_array().map_or(0,Vec::len),"artboards":proof["artboards"],"layers":proof["layers"],"warnings":warnings,"illustrator":proof["version"]}))
        })();
        // This directory was created by this call, with a unique name; never remove an input.
        let owned=std::fs::canonicalize(&temp).ok();
        let parent=temp.parent().and_then(|p|std::fs::canonicalize(p).ok());
        if let (Some(owned),Some(parent))=(owned,parent) {
            if owned.parent()==Some(parent.as_path()) && owned.file_name()==temp.file_name() {
                let _=std::fs::remove_dir_all(&owned);
            }
        }
        result
    },|app,result|{
        match &result {
            Ok(v)=>{let mut message=format!("已导出：{}\n颜色模式：{}；活文字框：{}；画板：{}。\n文件由本机 Illustrator 保存。请打开复核字体和复杂外观。",v["path"].as_str().unwrap_or(""),v["mode"].as_str().unwrap_or(""),v["textFrames"],v["artboards"]);
                if let Some(w)=v["warnings"].as_array().filter(|a|!a.is_empty()) {message.push_str("\n原生结构导出提示：\n");for note in w.iter().filter_map(Value::as_str){message.push_str(note);message.push('\n');}}
                crate::dialogs::confirm::tell(app,"可编辑 AI 导出完成",&message);app.status("可编辑 AI 导出完成。");}
            Err(e)=>app.status(format!("AI 导出失败：{e}")),
        }
        result
    })
}

#[cfg(windows)]
fn run_illustrator(script:&std::path::Path,manifest:&std::path::Path)->Result<(),String>{
    use std::os::windows::process::CommandExt;
    use std::process::{Command,Stdio};
    let failure=script.with_file_name("failure.txt");
    let code=r#"$ErrorActionPreference='Stop';try{$app=New-Object -ComObject Illustrator.Application;$js=[IO.File]::ReadAllText($env:LEO_AI_SCRIPT,[Text.Encoding]::UTF8);$prefix='var LEO_BUNDLE_PATH = '+(ConvertTo-Json -InputObject $env:LEO_AI_MANIFEST -Compress)+';';[void]$app.DoJavaScript($prefix+$js);[void][Runtime.InteropServices.Marshal]::ReleaseComObject($app);exit 0}catch{[IO.File]::WriteAllText($env:LEO_AI_FAILURE,$_.Exception.Message,[Text.Encoding]::UTF8);exit 1}"#;
    let mut child=Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-STA","-Command",code])
        .env("LEO_AI_SCRIPT",script).env("LEO_AI_MANIFEST",manifest).env("LEO_AI_FAILURE",&failure)
        .creation_flags(0x08000000).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()
        .map_err(|e|format!("无法启动 Illustrator 导出桥接：{e}"))?;
    let start=std::time::Instant::now();
    loop {
        if child.try_wait().map_err(|e|e.to_string())?.is_some(){break;}
        if start.elapsed()>std::time::Duration::from_secs(900){let _=child.kill();let _=child.wait();return Err("Illustrator 导出超时。请检查 Illustrator 是否有对话框或正在忙碌。".into());}
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
    let output=child.wait_with_output().map_err(|e|e.to_string())?;
    if output.status.success(){Ok(())}else{let why=std::fs::read_to_string(failure).unwrap_or_default();Err(format!("Illustrator 无法完成导出，请确认已安装并可正常启动。{}",why.chars().take(1500).collect::<String>()))}
}
#[cfg(not(windows))]
fn run_illustrator(_: &std::path::Path,_:&std::path::Path)->Result<(),String>{Err("需要 Windows Illustrator。".into())}

#[cfg(test)]
#[path="ai_export_tests.rs"]
mod leo_ai_export_tests;
