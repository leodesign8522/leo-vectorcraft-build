use super::*;
use vectorcraft_engine::Session;
use vectorcraft_doc::{ColorMode, NodeId};

fn fixture(mode:&str)->(Document,NodeId,NodeId){
    let mut s=Session::new();
    s.execute("file.new",&json!({"width":600,"height":400,"colorMode":mode})).unwrap();
    let shape=NodeId(s.execute("shape.rectangle",&json!({"x":40,"y":40,"width":100,"height":60})).unwrap()["id"].as_u64().unwrap());
    let color=if mode=="cmyk"{json!({"model":"cmyk","c":0.1,"m":0.2,"y":0.3,"k":0.4})}else{json!({"model":"rgb","r":0.2,"g":0.4,"b":0.6})};
    s.execute("paint.setFill",&json!({"color":color,"keepModel":true})).unwrap();
    s.execute("layer.new",&json!({"name":"Editable text"})).unwrap();
    let text=NodeId(s.execute("text.create",&json!({"x":60,"y":210,"text":"中文整段文字\nSecond paragraph","size":24,"area":{"width":210,"height":150,"fit":"none"},"color":color,"keepModel":true})).unwrap()["id"].as_u64().unwrap());
    (s.doc().unwrap().doc.as_ref().clone(),shape,text)
}

#[test]
fn leo_ai_export_tests_snapshot_keeps_source_colors_layers_and_full_area_text(){
    for mode in ["rgb","cmyk"]{
        let (doc,shape,text)=fixture(mode);let before=serde_json::to_value(&doc).unwrap();
        let p=prepare(&doc,"test").unwrap();assert_eq!(serde_json::to_value(&doc).unwrap(),before,"export must not mutate the source");
        assert_eq!(p.doc.color_mode,if mode=="cmyk"{ColorMode::Cmyk}else{ColorMode::Rgb});
        assert_eq!(p.doc.layers.len(),doc.layers.len());assert_eq!(p.doc.layers[1].name,doc.layers[1].name);
        assert_eq!(p.doc.node(shape),doc.node(shape));assert!(matches!(p.doc.node(text).unwrap().kind,NodeKind::Path{..}));
        assert_eq!(p.texts.len(),1);assert_eq!(p.texts[0]["node"],serde_json::to_value(doc.node(text).unwrap()).unwrap());
        assert_eq!(p.texts[0]["node"]["kind"]["kind"]["kind"],"area");
        let marker=p.texts[0]["marker"].as_str().unwrap();assert!(marker.encode_utf16().count()<=31);let swatch=p.doc.swatches.iter().find(|s|s.name==marker).unwrap();assert!(swatch.spot&&swatch.global);
        let b=p.doc.node(text).unwrap().geometric_bounds().unwrap();assert_eq!((b.x0,b.y0,b.width(),b.height()),(60.,210.,1.,1.));
        let (bytes,warnings)=vectorcraft_engine::cmd::fileio::encode_with_warnings(&p.doc,"ai",&json!({"editingData":true,"pdfCompatible":true})).unwrap();
        assert!(bytes.starts_with(b"%PDF-"));assert!(!warnings.iter().any(|w|w.contains("type is written as its outlines")));
    }
}

#[test]
fn leo_ai_export_tests_unsupported_text_fails_before_mutating_anything(){
    let (mut doc,_,id)=fixture("rgb");let NodeKind::Text(t)=&mut doc.node_mut(id).unwrap().kind else{panic!()};
    t.runs[0].style.font_axes.insert("wght".into(),700.0);
    let before=serde_json::to_value(&doc).unwrap();assert!(prepare(&doc,"bad").err().unwrap().contains("高级字符"));
    assert_eq!(serde_json::to_value(&doc).unwrap(),before);
}

#[test]
fn leo_ai_export_tests_mixed_runs_and_nested_parent_order_are_kept_in_manifest(){
    let (mut doc,shape,id)=fixture("cmyk");
    let NodeKind::Text(t)=&mut doc.node_mut(id).unwrap().kind else{panic!()};
    let mut run=t.runs[0].clone();run.text=" Bold 尾段".into();run.style.font_style="Bold".into();run.style.size=36.;t.runs.push(run);
    let original=doc.node(id).unwrap().clone();
    let group=doc.alloc_id();doc.wrap(&[shape,id],|children|Node::group(group,children)).unwrap();
    let before=serde_json::to_value(&doc).unwrap();let p=prepare(&doc,"nested").unwrap();
    assert_eq!(serde_json::to_value(&doc).unwrap(),before);
    assert_eq!(p.texts[0]["node"],serde_json::to_value(original).unwrap());
    assert_eq!(p.texts[0]["node"]["kind"]["runs"].as_array().unwrap().len(),2);
    assert_eq!(p.doc.node(group).unwrap().children().unwrap().len(),2);
}

#[test]
fn leo_ai_export_tests_empty_text_list_still_exports_geometry(){
    let (mut doc,shape,text)=fixture("rgb");doc.remove(text).unwrap();
    let p=prepare(&doc,"paths").unwrap();assert!(p.texts.is_empty());assert_eq!(p.doc.node(shape),doc.node(shape));
}

#[test]
fn leo_ai_export_tests_explicit_icc_is_not_silently_replaced(){
    for mode in ["rgb","cmyk"] {
        let (mut doc,_,_)=fixture(mode);
        if mode=="cmyk" {doc.color_profiles.cmyk=Some("Custom CMYK".into());}
        else {doc.color_profiles.rgb=Some("Custom RGB".into());}
        let before=serde_json::to_value(&doc).unwrap();
        assert!(prepare(&doc,"profile").err().unwrap().contains("ICC"));
        assert_eq!(serde_json::to_value(&doc).unwrap(),before);
    }
}

#[test]
fn leo_ai_export_tests_oversized_global_ink_names_fail_before_export(){
    let (mut doc,_,_)=fixture("rgb");
    doc.swatches.push(Swatch{name:"a".repeat(32),paint:Paint::solid(Color::rgb(1.,0.,0.)),global:true,spot:true});
    let before=serde_json::to_value(&doc).unwrap();
    assert!(prepare(&doc,"long").err().unwrap().contains("31"));
    assert_eq!(serde_json::to_value(&doc).unwrap(),before);
}
