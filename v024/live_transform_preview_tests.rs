
/// A deliberately stalled worker makes this deterministic: a transform must not
/// show that worker's old bitmap under the new document's selection overlays.
#[test]
fn leo_render_live_move_and_scale_do_not_wait_for_old_worker_pixels() {
    for tool in ["selection", "scale"] {
        for last_ms in [1.0, 40.0] {
            let mut app=VectorcraftApp::new(Session::new(),Default::default());
            app.run("file.new",json!({"width":400,"height":300})).unwrap();
            let id=app.run("shape.rectangle",json!({"x":100,"y":100,"width":80,"height":60})).unwrap()["id"].as_u64().unwrap();
            app.canvas.worker_started=true;
            let ctx=egui::Context::default();
            frame_output(&mut app,&ctx).1.clear();
            let (worker,_jobs,_done)=crate::render_worker::Worker::test_channel();
            app.canvas.worker=Some(worker);
            app.canvas.from_worker=true;app.canvas.last_ms=last_ms;
            app.select_tool(tool);
            let view=app.view_info();
            let start=if tool=="selection" {Point::new(140.,130.)} else {Point::new(180.,160.)};
            app.session.pointer(&PointerEvent::new(PointerKind::Down,start.x,start.y),view).unwrap();
            let ends=if tool=="selection" {[Point::new(340.,130.),Point::new(140.,230.)]}
                else {[Point::new(156.,142.),Point::new(164.,148.)]};
            for end in ends {
                app.session.pointer(&PointerEvent::new(PointerKind::Drag,end.x,end.y),view).unwrap();
                assert!(app.session.in_interaction(),"{tool} starts a native transform");
                let (_,mut delta)=frame_output(&mut app,&ctx);
                let key=app.canvas.key.as_ref().unwrap();
                assert_eq!(key.revision,app.session.doc().unwrap().revision,"{tool}: artwork and overlays must be the same revision");
                assert!(!app.canvas.from_worker);
                let art=app.canvas.texture.as_ref().unwrap().id();
                let upload=delta.set.iter().find(|e|e.0.eq(&art)).and_then(|e|e.1.last()).expect("each transform uploads its current pixels");
                assert!(upload.pos.is_none(),"a complete live image, never a partial write into a draft");
                let egui::ImageData::Color(image)=&upload.image;
                if last_ms>8. {assert!(app.canvas.live_draft);assert!(image.size[0]*image.size[1]<=262144);}
                else {assert!(!app.canvas.live_draft);assert_eq!(image.size,[key.w as usize,key.h as usize]);}
                let xf=Xf::new(app.canvas_rect.unwrap(),app.view().unwrap());
                let alpha=|p:Point| {
                    let p=xf.to_screen(p);let r=app.canvas_rect.unwrap();
                    let x=((p.x-r.left())/r.width()*image.size[0] as f32).floor() as usize;
                    let y=((p.y-r.top())/r.height()*image.size[1] as f32).floor() as usize;
                    image.pixels.get(y*image.size[0]+x).map_or(0,|p|p.a())
                };
                let b=app.session.doc().unwrap().doc.node(vectorcraft_doc::NodeId(id)).unwrap().geometric_bounds().unwrap();
                assert!(alpha(b.center())>0,"the object is drawn at its current position");
                let old=if tool=="selection" {Point::new(140.,130.)} else {Point::new(108.,108.)};
                assert_eq!(alpha(old),0,"{tool}: the old position must not leave a trail");
                delta.clear();
            }
        }
    }
}

#[test]
fn leo_render_draft_restores_full_resolution_and_rejects_late_drag_frame() {
    let mut app=VectorcraftApp::new(Session::new(),Default::default());
    app.run("file.new",json!({"width":400,"height":300})).unwrap();
    app.run("shape.rectangle",json!({"x":100,"y":100,"width":80,"height":60})).unwrap();
    app.canvas.worker_started=true;
    let ctx=egui::Context::default();frame_output(&mut app,&ctx).1.clear();
    let (worker,jobs,done)=crate::render_worker::Worker::test_channel();app.canvas.worker=Some(worker);
    app.canvas.from_worker=true;app.canvas.last_ms=40.;app.select_tool("selection");
    let view=app.view_info();
    app.session.pointer(&PointerEvent::new(PointerKind::Down,140.,130.),view).unwrap();
    app.session.pointer(&PointerEvent::new(PointerKind::Drag,240.,150.),view).unwrap();
    frame_output(&mut app,&ctx).1.clear();assert!(app.canvas.live_draft);
    let draft=app.canvas.key.clone().unwrap();let draft_doc=app.session.doc().unwrap().doc.clone();
    app.session.pointer(&PointerEvent::new(PointerKind::Up,240.,150.),view).unwrap();
    frame_output(&mut app,&ctx).1.clear();
    let job=jobs.try_recv().expect("release queues a full-resolution still frame");
    assert!(!job.key.interacting);assert_eq!((job.w,job.h),(job.key.w,job.key.h));
    let mut renderer=vectorcraft_render::Renderer::new();
    let img=renderer.render(&job.doc,job.w,job.h,job.view,&job.opts);
    done.send(crate::render_worker::Done{key:job.key,doc:job.doc,opts:job.opts,img,ms:2.,moved:0,reused:false}).unwrap();
    let (_,mut delta)=frame_output(&mut app,&ctx);
    assert!(!app.canvas.live_draft);
    let key=app.canvas.key.clone().unwrap();assert!(!key.interacting);
    let art=app.canvas.texture.as_ref().unwrap().id();
    let upload=delta.set.iter().find(|e|e.0.eq(&art)).and_then(|e|e.1.last()).expect("full-resolution texture restored");
    let egui::ImageData::Color(img)=&upload.image;
    assert_eq!(img.size,[key.w as usize,key.h as usize]);assert!(upload.pos.is_none());delta.clear();
    let mut late=draft;late.revision=key.revision;
    done.send(crate::render_worker::Done{key:late,doc:draft_doc,opts:Default::default(),
        img:vectorcraft_render::Rendered{width:1,height:1,pixels:vec![255;4]},ms:1.,moved:0,reused:false}).unwrap();
    let (_,mut delta)=frame_output(&mut app,&ctx);
    assert!(!delta.set.iter().any(|e|e.0.eq(&art)),"a late drag preview must not replace the settled full frame");
    assert!(!app.canvas.key.as_ref().unwrap().interacting);delta.clear();
}

#[test]
fn leo_render_preview_budget_is_bounded_without_reducing_light_drawings() {
    assert_eq!(live_transform_preview::dimensions(1920,1080,1.),(1920,1080));
    for (w,h) in [(800,600),(3840,2160),(65535,65535),(1,65535)] {
        let (pw,ph)=live_transform_preview::dimensions(w,h,40.);
        assert!(pw>0 && ph>0 && pw<=w && ph<=h);assert!(u64::from(pw)*u64::from(ph)<=262144);
    }
}
