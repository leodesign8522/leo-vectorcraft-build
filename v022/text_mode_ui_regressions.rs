#[cfg(test)]
mod leo_text_mode_ui_tests {
    use super::*;
    use vectorcraft_doc::{AreaFit,NodeId};
    fn app()->VectorcraftApp {let mut app=VectorcraftApp::new(vectorcraft_engine::Session::new(),Default::default());app.run("file.new",json!({"width":1000,"height":1000})).unwrap();app}
    fn make(a:&mut VectorcraftApp,x:f64)->u64 {a.run("text.create",json!({"x":x,"y":100,"text":"B5","area":{"width":200,"height":500}})).unwrap()["id"].as_u64().unwrap()}
    #[test]
    fn leo_text_mode_ui_selected_mixed_and_no_selection_states() {
        let mut a=app();let one=make(&mut a,100.0);assert_eq!(leo_area_height_state(&a),(true,Some(AreaFit::AutoHeight)));
        let painted=crate::tests_labels::painted_text(&mut a,|a,ui|show(a,ui));assert!(painted.contains("✓ 自动贴合高度"));assert!(painted.contains("当前：自动贴合高度"));
        let two=make(&mut a,400.0);a.run("text.areaOptions",json!({"fit":"none"})).unwrap();assert_eq!(leo_area_height_state(&a),(true,Some(AreaFit::None)));
        a.run("select.set",json!({"ids":[one,two]})).unwrap();assert_eq!(leo_area_height_state(&a),(true,None));
        let painted=crate::tests_labels::painted_text(&mut a,|a,ui|show(a,ui));assert!(painted.contains("混合状态"));
        a.run("object.group",json!({})).unwrap();assert_eq!(leo_area_height_state(&a),(true,None));
        a.run("text.areaOptions",json!({"fit":"autoHeight"})).unwrap();assert_eq!(leo_area_height_state(&a),(true,Some(AreaFit::AutoHeight)));
        a.run("edit.undo",json!({})).unwrap();assert_eq!(leo_area_height_state(&a),(true,None));
        a.run("select.none",json!({})).unwrap();assert_eq!(leo_area_height_state(&a),(false,None));
        assert!(a.session.active().unwrap().doc.node(NodeId(one)).is_some());
    }
    #[test]
    fn leo_text_mode_ui_preferences_migrate_once_and_preserve_opaque_values() {
        let mut a=app();a.ui.engine_prefs=json!({"autoSizeAreaType":false,"optionalFutureSetting":{"kept":true}});
        crate::prefs_dialog::restore(&mut a);assert!(a.session.prefs.auto_size_area_type);assert!(a.session.prefs.leo_text_defaults_v1);
        assert_eq!(a.ui.engine_prefs["optionalFutureSetting"]["kept"],true);
        a.ui.engine_prefs["autoSizeAreaType"]=json!(false);crate::prefs_dialog::restore(&mut a);assert!(!a.session.prefs.auto_size_area_type);
    }
}
